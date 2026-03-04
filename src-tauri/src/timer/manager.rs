use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use tauri::AppHandle;
use tauri::Emitter;

use crate::error::{command_error, CommandError};
use crate::persistence::settings::TimerSettings;
use crate::persistence::store;

use super::types::{PersistedTimerState, TimerSettingsDto, TimerStateDto, TimerStatus};

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
            act_timer_enabled: settings.act_timer_enabled,
            campaign_timer_enabled: settings.campaign_timer_enabled,
        })
    }

    pub fn set_settings(
        app: &AppHandle,
        act_timer_enabled: bool,
        campaign_timer_enabled: bool,
    ) -> Result<TimerSettingsDto, CommandError> {
        let settings = TimerSettings {
            act_timer_enabled,
            campaign_timer_enabled,
        };
        store::set_value(app, TimerSettings::STORE_KEY, &settings)?;
        Ok(TimerSettingsDto {
            act_timer_enabled: settings.act_timer_enabled,
            campaign_timer_enabled: settings.campaign_timer_enabled,
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
        guard.last_tick = if guard.state.status == TimerStatus::Running {
            Some(Instant::now())
        } else {
            None
        };

        Ok(TimerStateDto::from(&guard.state))
    }

    pub fn get_state(&self) -> Result<TimerStateDto, CommandError> {
        let mut guard = self
            .inner
            .lock()
            .map_err(|_| command_error("timer_state_poisoned", "Timer state poisoned"))?;

        flush_elapsed(&mut guard);
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
