use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use uuid::Uuid;

use tauri::AppHandle;
use tauri::Emitter;

use crate::error::{command_error, CommandError};
use crate::logging;
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
            warn_when_paused: settings.warn_when_paused,
        })
    }

    pub fn set_settings(
        app: &AppHandle,
        enabled: bool,
        display_act_timer: bool,
        display_campaign_timer: bool,
        warn_when_paused: bool,
    ) -> Result<TimerSettingsDto, CommandError> {
        let settings = TimerSettings {
            schema_version: 1,
            enabled,
            display_act_timer,
            display_campaign_timer,
            warn_when_paused,
        };
        store::set_value(app, TimerSettings::STORE_KEY, &settings)?;
        Ok(TimerSettingsDto {
            schema_version: settings.schema_version,
            enabled: settings.enabled,
            display_act_timer: settings.display_act_timer,
            display_campaign_timer: settings.display_campaign_timer,
            warn_when_paused: settings.warn_when_paused,
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

        logging::info(
            "timer",
            &format!(
                "load_state: persisted status={:?} act_index={} act_ms={} campaign_ms={}",
                persisted.status,
                persisted.current_act_index,
                persisted.current_act_elapsed_ms,
                persisted.campaign_elapsed_ms
            ),
        );

        guard.state = persisted;
        if guard.state.status == TimerStatus::Running {
            logging::info("timer", "load_state: was running, transitioning to paused");
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
            logging::warn(
                "timer",
                &format!("start: rejected, current status={:?}", guard.state.status),
            );
            return Err(command_error(
                "timer_invalid_transition",
                "Timer can only be started from idle state",
            ));
        }

        guard.state.status = TimerStatus::Running;
        guard.state.run_id = Some(Uuid::new_v4().to_string());
        guard.last_tick = Some(Instant::now());

        logging::info(
            "timer",
            &format!("start: timer started from idle, run_id={:?}", guard.state.run_id),
        );
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
            logging::warn(
                "timer",
                &format!("pause: rejected, current status={:?}", guard.state.status),
            );
            return Err(command_error(
                "timer_invalid_transition",
                "Timer can only be paused from running state",
            ));
        }

        flush_elapsed(&mut guard);
        guard.state.status = TimerStatus::Paused;
        guard.last_tick = None;

        logging::info(
            "timer",
            &format!(
                "pause: act_index={} act_ms={} campaign_ms={}",
                guard.state.current_act_index,
                guard.state.current_act_elapsed_ms,
                guard.state.campaign_elapsed_ms
            ),
        );
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
            logging::warn(
                "timer",
                &format!("resume: rejected, current status={:?}", guard.state.status),
            );
            return Err(command_error(
                "timer_invalid_transition",
                "Timer can only be resumed from paused state",
            ));
        }

        guard.state.status = TimerStatus::Running;
        guard.last_tick = Some(Instant::now());

        logging::info(
            "timer",
            &format!(
                "resume: act_index={} act_ms={} campaign_ms={}",
                guard.state.current_act_index,
                guard.state.current_act_elapsed_ms,
                guard.state.campaign_elapsed_ms
            ),
        );
        let dto = TimerStateDto::from(&guard.state);
        save_state_inner(app, &guard.state);
        Ok(dto)
    }

    pub fn reset(&self, app: &AppHandle) -> Result<TimerStateDto, CommandError> {
        let mut guard = self
            .inner
            .lock()
            .map_err(|_| command_error("timer_state_poisoned", "Timer state poisoned"))?;

        logging::info(
            "timer",
            &format!(
                "reset: from status={:?} act_index={} campaign_ms={}",
                guard.state.status,
                guard.state.current_act_index,
                guard.state.campaign_elapsed_ms
            ),
        );

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

        logging::info(
            "timer",
            &format!(
                "notify_act_completed: requested_act={} timer_status={:?} timer_act_index={}",
                completed_act_index, guard.state.status, guard.state.current_act_index
            ),
        );

        if guard.state.status != TimerStatus::Running {
            logging::warn(
                "timer",
                &format!(
                    "notify_act_completed: skipped, timer not running (status={:?})",
                    guard.state.status
                ),
            );
            return Ok(());
        }

        if completed_act_index != guard.state.current_act_index {
            logging::warn(
                "timer",
                &format!(
                    "notify_act_completed: act index mismatch, guide says {} but timer is at {}",
                    completed_act_index, guard.state.current_act_index
                ),
            );
            return Ok(());
        }

        flush_elapsed(&mut guard);

        if completed_act_index < ACT_COUNT {
            guard.state.act_elapsed_ms[completed_act_index] = guard.state.current_act_elapsed_ms;
        }

        let next_act = completed_act_index + 1;
        guard.state.current_act_index = next_act;
        guard.state.current_act_elapsed_ms = 0;

        logging::info(
            "timer",
            &format!(
                "notify_act_completed: act {} completed with {}ms, advancing to act {}",
                completed_act_index + 1,
                guard.state.act_elapsed_ms.get(completed_act_index).copied().unwrap_or(0),
                next_act + 1
            ),
        );

        let dto = TimerStateDto::from(&guard.state);
        save_state_inner(app, &guard.state);

        if let Err(err) = app.emit(TIMER_STATE_UPDATED_EVENT, &dto) {
            logging::error("timer", &format!("notify_act_completed: failed to emit update: {err}"));
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

        let run_id = guard
            .state
            .run_id
            .clone()
            .unwrap_or_else(|| Uuid::new_v4().to_string());

        let run = SavedRun {
            schema_version: 1,
            id: run_id.clone(),
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

        if guard.state.run_id.is_none() {
            guard.state.run_id = Some(run_id);
            save_state_inner(app, &guard.state);
        }

        let mut data = load_saved_runs_data(app)?;
        if let Some(existing) = data.runs.iter_mut().find(|r| r.id == run.id) {
            *existing = run.clone();
        } else {
            data.runs.push(run.clone());
        }
        save_saved_runs_data(app, &data)?;

        Ok(SavedRunDto::from(&run))
    }

    pub fn load_runs(app: &AppHandle) -> Result<Vec<SavedRunDto>, CommandError> {
        let data = load_saved_runs_data(app)?;
        Ok(data.runs.iter().map(SavedRunDto::from).collect())
    }

    pub fn edit_run(
        app: &AppHandle,
        run_id: String,
        league: String,
        hardcore: bool,
        ssf: bool,
        private_league: bool,
        character: String,
        character_class: String,
        run_details: String,
    ) -> Result<SavedRunDto, CommandError> {
        let mut data = load_saved_runs_data(app)?;
        let run = data
            .runs
            .iter_mut()
            .find(|r| r.id == run_id)
            .ok_or_else(|| {
                command_error(
                    "saved_run_not_found",
                    format!("No saved run with id '{run_id}'"),
                )
            })?;

        run.league = league;
        run.hardcore = hardcore;
        run.ssf = ssf;
        run.private_league = private_league;
        run.character = character;
        run.character_class = character_class;
        run.run_details = run_details;
        run.saved_at = now_epoch_ms();

        let dto = SavedRunDto::from(&*run);
        save_saved_runs_data(app, &data)?;
        Ok(dto)
    }

    pub fn get_run(app: &AppHandle, run_id: String) -> Result<Option<SavedRunDto>, CommandError> {
        let data = load_saved_runs_data(app)?;
        let found = data.runs.iter().find(|r| r.id == run_id).map(SavedRunDto::from);
        Ok(found)
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
        let preserved_run_id = saved.id.clone();

        let mut guard = self
            .inner
            .lock()
            .map_err(|_| command_error("timer_state_poisoned", "Timer state poisoned"))?;

        guard.state = restored;
        guard.state.run_id = Some(preserved_run_id);
        guard.last_tick = None;

        let dto = TimerStateDto::from(&guard.state);
        save_state_inner(app, &guard.state);
        Ok(dto)
    }

    pub fn start_auto_save(self: &Arc<Self>, app: &AppHandle) {
        if self.auto_save_running.swap(true, Ordering::SeqCst) {
            return;
        }

        logging::info("timer", "start_auto_save: spawning auto-save thread");

        let manager = Arc::clone(self);
        let app_handle = app.clone();

        std::thread::spawn(move || {
            loop {
                std::thread::sleep(AUTO_SAVE_INTERVAL);

                let state_snapshot = {
                    let mut guard = match manager.inner.lock() {
                        Ok(g) => g,
                        Err(_) => {
                            logging::error("timer", "auto_save: mutex poisoned, stopping thread");
                            break;
                        }
                    };

                    if guard.state.status != TimerStatus::Running {
                        continue;
                    }

                    flush_elapsed(&mut guard);
                    guard.state.clone()
                };

                logging::info(
                    "timer",
                    &format!(
                        "auto_save: act_index={} act_ms={} campaign_ms={}",
                        state_snapshot.current_act_index,
                        state_snapshot.current_act_elapsed_ms,
                        state_snapshot.campaign_elapsed_ms
                    ),
                );

                save_state_inner(&app_handle, &state_snapshot);

                let dto = TimerStateDto::from(&state_snapshot);
                if let Err(err) = app_handle.emit(TIMER_STATE_UPDATED_EVENT, &dto) {
                    logging::error("timer", &format!("auto_save: failed to emit update: {err}"));
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
        logging::error("timer", &format!("save_state_inner: persist failed: {:?}", err));
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
        run_id: None,
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
