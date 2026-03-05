use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use tauri::AppHandle;
use tauri::Emitter;

use crate::error::{command_error, CommandError};
use crate::persistence::settings::TimerSettings;
use crate::persistence::store;

use super::types::{
    ActRun, ActRunStatus, PersistedTimerState, SavedRun, SavedRunDto, SavedRunStatus,
    SavedRunsData, TimerSettingsDto, TimerStateDto, TimerStatus,
};

const AUTO_SAVE_INTERVAL: Duration = Duration::from_secs(5);
const TIMER_STATE_UPDATED_EVENT: &str = "timer_state_updated";
const ACT_COUNT: usize = 10;

struct TimerInner {
    state: PersistedTimerState,
    last_tick: Option<Instant>,
}

pub struct TimerManager {
    inner: Mutex<TimerInner>,
    auto_save_running: AtomicBool,
}

impl Default for TimerManager {
    fn default() -> Self {
        Self {
            inner: Mutex::new(TimerInner {
                state: PersistedTimerState::default(),
                last_tick: None,
            }),
            auto_save_running: AtomicBool::new(false),
        }
    }
}

impl TimerManager {
    fn load_settings(app: &AppHandle) -> Result<TimerSettings, CommandError> {
        let maybe = store::get_optional::<TimerSettings>(app, TimerSettings::STORE_KEY)?;
        Ok(maybe.unwrap_or_default())
    }

    pub fn get_settings(app: &AppHandle) -> Result<TimerSettingsDto, CommandError> {
        let settings = Self::load_settings(app)?;
        Ok(TimerSettingsDto {
            schema_version: settings.schema_version,
            enabled: settings.enabled,
            display_act_timer: settings.display_act_timer,
            display_campaign_timer: settings.display_campaign_timer,
        })
    }

    pub fn set_settings(
        app: &AppHandle,
        enabled: bool,
        display_act_timer: bool,
        display_campaign_timer: bool,
    ) -> Result<TimerSettingsDto, CommandError> {
        let settings = TimerSettings {
            schema_version: 1,
            enabled,
            display_act_timer,
            display_campaign_timer,
        };
        store::set_value(app, TimerSettings::STORE_KEY, &settings)?;
        Ok(TimerSettingsDto {
            schema_version: settings.schema_version,
            enabled: settings.enabled,
            display_act_timer: settings.display_act_timer,
            display_campaign_timer: settings.display_campaign_timer,
        })
    }

    pub fn load_state(&self, app: &AppHandle) -> Result<TimerStateDto, CommandError> {
        let mut guard = self
            .inner
            .lock()
            .map_err(|_| command_error("timer_state_poisoned", "Timer state poisoned"))?;

        let persisted =
            store::get_optional::<PersistedTimerState>(app, PersistedTimerState::STORE_KEY)?
                .unwrap_or_default();

        guard.state = persisted;
        if guard.state.status == TimerStatus::Running {
            guard.state.status = TimerStatus::Paused;
            save_state_inner(app, &guard.state);
        }
        guard.last_tick = None;

        Ok(TimerStateDto::from(&guard.state))
    }

    pub fn start(&self, app: &AppHandle) -> Result<TimerStateDto, CommandError> {
        let mut guard = self
            .inner
            .lock()
            .map_err(|_| command_error("timer_state_poisoned", "Timer state poisoned"))?;

        if guard.state.status != TimerStatus::Idle {
            return Err(command_error(
                "timer_invalid_transition",
                "Timer can only be started from idle state",
            ));
        }

        guard.state.status = TimerStatus::Running;
        guard.last_tick = Some(Instant::now());

        let dto = TimerStateDto::from(&guard.state);
        save_state_inner(app, &guard.state);
        Ok(dto)
    }

    pub fn pause(&self, app: &AppHandle) -> Result<TimerStateDto, CommandError> {
        let mut guard = self
            .inner
            .lock()
            .map_err(|_| command_error("timer_state_poisoned", "Timer state poisoned"))?;

        if guard.state.status != TimerStatus::Running {
            return Err(command_error(
                "timer_invalid_transition",
                "Timer can only be paused from running state",
            ));
        }

        flush_elapsed(&mut guard);
        guard.state.status = TimerStatus::Paused;
        guard.last_tick = None;

        let dto = TimerStateDto::from(&guard.state);
        save_state_inner(app, &guard.state);
        Ok(dto)
    }

    pub fn resume(&self, app: &AppHandle) -> Result<TimerStateDto, CommandError> {
        let mut guard = self
            .inner
            .lock()
            .map_err(|_| command_error("timer_state_poisoned", "Timer state poisoned"))?;

        if guard.state.status != TimerStatus::Paused {
            return Err(command_error(
                "timer_invalid_transition",
                "Timer can only be resumed from paused state",
            ));
        }

        guard.state.status = TimerStatus::Running;
        guard.last_tick = Some(Instant::now());

        let dto = TimerStateDto::from(&guard.state);
        save_state_inner(app, &guard.state);
        Ok(dto)
    }

    pub fn reset(&self, app: &AppHandle) -> Result<TimerStateDto, CommandError> {
        let mut guard = self
            .inner
            .lock()
            .map_err(|_| command_error("timer_state_poisoned", "Timer state poisoned"))?;

        guard.state = PersistedTimerState::default();
        guard.last_tick = None;

        let dto = TimerStateDto::from(&guard.state);
        save_state_inner(app, &guard.state);
        Ok(dto)
    }

    pub fn notify_act_completed(
        &self,
        app: &AppHandle,
        completed_act_index: usize,
    ) -> Result<(), CommandError> {
        let mut guard = self
            .inner
            .lock()
            .map_err(|_| command_error("timer_state_poisoned", "Timer state poisoned"))?;

        if guard.state.status != TimerStatus::Running {
            return Ok(());
        }

        if completed_act_index != guard.state.current_act_index {
            return Ok(());
        }

        flush_elapsed(&mut guard);

        if completed_act_index < ACT_COUNT {
            guard.state.act_elapsed_ms[completed_act_index] = guard.state.current_act_elapsed_ms;
        }

        let next_act = completed_act_index + 1;
        guard.state.current_act_index = next_act;
        guard.state.current_act_elapsed_ms = 0;

        let dto = TimerStateDto::from(&guard.state);
        save_state_inner(app, &guard.state);

        if let Err(err) = app.emit(TIMER_STATE_UPDATED_EVENT, &dto) {
            eprintln!("Failed to emit timer state update: {err}");
        }

        Ok(())
    }

    pub fn save_run(
        &self,
        app: &AppHandle,
        league: String,
        hardcore: bool,
        ssf: bool,
        private_league: bool,
        character: String,
        character_class: String,
        run_details: String,
    ) -> Result<SavedRunDto, CommandError> {
        let mut guard = self
            .inner
            .lock()
            .map_err(|_| command_error("timer_state_poisoned", "Timer state poisoned"))?;

        if guard.state.status == TimerStatus::Running {
            flush_elapsed(&mut guard);
        }

        let act_runs = build_act_runs(&guard.state);
        let all_completed = act_runs.iter().all(|a| a.status == ActRunStatus::Completed);
        let run_status = if all_completed {
            SavedRunStatus::Completed
        } else {
            SavedRunStatus::InProgress
        };
        let now_ms = now_epoch_ms();

        let run = SavedRun {
            schema_version: 1,
            id: now_ms.to_string(),
            league,
            hardcore,
            ssf,
            private_league,
            character,
            character_class,
            run_details,
            status: run_status,
            act_runs,
            campaign_elapsed_ms: guard.state.campaign_elapsed_ms,
            saved_at: now_ms,
        };

        let mut data = load_saved_runs_data(app)?;
        data.runs.push(run.clone());
        save_saved_runs_data(app, &data)?;

        Ok(SavedRunDto::from(&run))
    }

    pub fn load_runs(app: &AppHandle) -> Result<Vec<SavedRunDto>, CommandError> {
        let data = load_saved_runs_data(app)?;
        Ok(data.runs.iter().map(SavedRunDto::from).collect())
    }

    pub fn delete_run(app: &AppHandle, run_id: String) -> Result<Vec<SavedRunDto>, CommandError> {
        let mut data = load_saved_runs_data(app)?;
        let before_len = data.runs.len();
        data.runs.retain(|r| r.id != run_id);

        if data.runs.len() == before_len {
            return Err(command_error(
                "saved_run_not_found",
                format!("No saved run with id '{run_id}'"),
            ));
        }

        save_saved_runs_data(app, &data)?;
        Ok(data.runs.iter().map(SavedRunDto::from).collect())
    }

    pub fn continue_run(
        &self,
        app: &AppHandle,
        run_id: String,
    ) -> Result<TimerStateDto, CommandError> {
        let data = load_saved_runs_data(app)?;
        let saved = data
            .runs
            .iter()
            .find(|r| r.id == run_id)
            .ok_or_else(|| {
                command_error(
                    "saved_run_not_found",
                    format!("No saved run with id '{run_id}'"),
                )
            })?;

        let restored = restore_timer_state(saved);

        let mut guard = self
            .inner
            .lock()
            .map_err(|_| command_error("timer_state_poisoned", "Timer state poisoned"))?;

        guard.state = restored;
        guard.last_tick = None;

        let dto = TimerStateDto::from(&guard.state);
        save_state_inner(app, &guard.state);
        Ok(dto)
    }

    pub fn start_auto_save(self: &Arc<Self>, app: &AppHandle) {
        if self.auto_save_running.swap(true, Ordering::SeqCst) {
            return;
        }

        let manager = Arc::clone(self);
        let app_handle = app.clone();

        std::thread::spawn(move || {
            loop {
                std::thread::sleep(AUTO_SAVE_INTERVAL);

                let state_snapshot = {
                    let mut guard = match manager.inner.lock() {
                        Ok(g) => g,
                        Err(_) => break,
                    };

                    if guard.state.status != TimerStatus::Running {
                        continue;
                    }

                    flush_elapsed(&mut guard);
                    guard.state.clone()
                };

                save_state_inner(&app_handle, &state_snapshot);

                let dto = TimerStateDto::from(&state_snapshot);
                if let Err(err) = app_handle.emit(TIMER_STATE_UPDATED_EVENT, &dto) {
                    eprintln!("Timer auto-save: failed to emit update: {err}");
                }
            }
        });
    }
}

fn flush_elapsed(inner: &mut TimerInner) {
    let Some(last_tick) = inner.last_tick else {
        return;
    };

    let now = Instant::now();
    let delta_ms = now.duration_since(last_tick).as_millis() as u64;
    inner.state.current_act_elapsed_ms += delta_ms;
    inner.state.campaign_elapsed_ms += delta_ms;
    inner.last_tick = Some(now);
}

fn save_state_inner(app: &AppHandle, state: &PersistedTimerState) {
    if let Err(err) = store::set_value(app, PersistedTimerState::STORE_KEY, state) {
        eprintln!("Failed to persist timer state: {:?}", err);
    }
}

fn build_act_runs(state: &PersistedTimerState) -> Vec<ActRun> {
    (0..ACT_COUNT)
        .map(|i| {
            let status = if i < state.current_act_index {
                ActRunStatus::Completed
            } else if i == state.current_act_index && state.status != TimerStatus::Idle {
                ActRunStatus::InProgress
            } else {
                ActRunStatus::Pending
            };

            let elapsed_ms = if i == state.current_act_index && state.status != TimerStatus::Idle {
                state.current_act_elapsed_ms
            } else {
                state.act_elapsed_ms.get(i).copied().unwrap_or(0)
            };

            ActRun {
                act_name: format!("Act {}", i + 1),
                elapsed_ms,
                status,
            }
        })
        .collect()
}

fn load_saved_runs_data(app: &AppHandle) -> Result<SavedRunsData, CommandError> {
    let maybe = store::get_optional::<SavedRunsData>(app, SavedRunsData::STORE_KEY)?;
    Ok(maybe.unwrap_or_default())
}

fn save_saved_runs_data(app: &AppHandle, data: &SavedRunsData) -> Result<(), CommandError> {
    store::set_value(app, SavedRunsData::STORE_KEY, data)
}

fn restore_timer_state(saved: &SavedRun) -> PersistedTimerState {
    let mut act_elapsed_ms = vec![0u64; ACT_COUNT];
    let mut current_act_index = ACT_COUNT;
    let mut current_act_elapsed_ms = 0u64;

    for (i, act) in saved.act_runs.iter().enumerate().take(ACT_COUNT) {
        match act.status {
            ActRunStatus::Completed => {
                act_elapsed_ms[i] = act.elapsed_ms;
            }
            ActRunStatus::InProgress => {
                current_act_index = i;
                current_act_elapsed_ms = act.elapsed_ms;
            }
            ActRunStatus::Pending => {}
        }
    }

    if current_act_index == ACT_COUNT {
        current_act_index = saved
            .act_runs
            .iter()
            .take(ACT_COUNT)
            .position(|a| a.status != ActRunStatus::Completed)
            .unwrap_or(ACT_COUNT);
    }

    PersistedTimerState {
        schema_version: 1,
        status: TimerStatus::Paused,
        current_act_index,
        act_elapsed_ms,
        current_act_elapsed_ms,
        campaign_elapsed_ms: saved.campaign_elapsed_ms,
    }
}

fn now_epoch_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
