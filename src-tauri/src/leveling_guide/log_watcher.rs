use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use tauri::AppHandle;
use tauri::Emitter;

use crate::leveling_guide::progress::save_leveling_guide_progress;
use crate::leveling_guide::LevelingGuideManager;
use crate::logging;
use crate::timer::TimerManager;

const POLL_INTERVAL: Duration = Duration::from_millis(500);
const LEVELING_GUIDE_PAGE_UPDATED_EVENT: &str = "leveling_guide_page_updated";

pub(crate) struct LogWatcherHandle {
    stop_flag: Arc<AtomicBool>,
}

impl LogWatcherHandle {
    pub(crate) fn stop(&self) {
        self.stop_flag.store(true, Ordering::Relaxed);
    }
}

impl Drop for LogWatcherHandle {
    fn drop(&mut self) {
        self.stop();
    }
}

fn parse_area_id_from_line(line: &str) -> Option<&str> {
    let generating_index = line.find("Generating level ")?;
    let after_generating = &line[generating_index..];

    let area_start_marker = "area \"";
    let area_marker_index = after_generating.find(area_start_marker)?;
    let area_id_start = area_marker_index + area_start_marker.len();
    let remaining = &after_generating[area_id_start..];

    let area_id_end = remaining.find('"')?;
    let area_id = &remaining[..area_id_end];

    if area_id.is_empty() {
        return None;
    }

    Some(area_id)
}

fn seek_to_end(file: &mut std::fs::File) -> std::io::Result<u64> {
    file.seek(SeekFrom::End(0))
}

fn read_new_content(file: &mut std::fs::File) -> std::io::Result<String> {
    let mut buffer = String::new();
    file.read_to_string(&mut buffer)?;
    Ok(buffer)
}

fn extract_last_area_id(content: &str) -> Option<String> {
    let mut last_area_id: Option<String> = None;

    for line in content.lines() {
        if let Some(area_id) = parse_area_id_from_line(line) {
            last_area_id = Some(area_id.to_string());
        }
    }

    last_area_id
}

pub(crate) fn spawn_log_watcher(
    app: &AppHandle,
    manager: &Arc<LevelingGuideManager>,
    timer_manager: &Arc<TimerManager>,
    log_path: &str,
) -> Result<LogWatcherHandle, String> {
    let path = Path::new(log_path);
    if !path.exists() {
        logging::warn("log_watcher", &format!("spawn: client log not found: {log_path}"));
        return Err(format!("Client log file not found: {log_path}"));
    }

    let stop_flag = Arc::new(AtomicBool::new(false));
    let handle = LogWatcherHandle {
        stop_flag: Arc::clone(&stop_flag),
    };

    let app_handle = app.clone();
    let manager = Arc::clone(manager);
    let timer_manager = Arc::clone(timer_manager);
    let log_path_owned = log_path.to_string();

    logging::info("log_watcher", &format!("spawn: starting watcher for {log_path}"));

    std::thread::spawn(move || {
        if let Err(err) = poll_loop(&app_handle, &manager, &timer_manager, &log_path_owned, &stop_flag) {
            logging::error("log_watcher", &format!("poll_loop exited with error: {err}"));
        }
    });

    Ok(handle)
}

fn poll_loop(
    app: &AppHandle,
    manager: &LevelingGuideManager,
    timer_manager: &TimerManager,
    log_path: &str,
    stop_flag: &AtomicBool,
) -> Result<(), String> {
    let mut file =
        std::fs::File::open(log_path).map_err(|e| format!("Failed to open log file: {e}"))?;

    seek_to_end(&mut file).map_err(|e| format!("Failed to seek to end of log file: {e}"))?;

    while !stop_flag.load(Ordering::Relaxed) {
        std::thread::sleep(POLL_INTERVAL);

        if stop_flag.load(Ordering::Relaxed) {
            break;
        }

        let content = match read_new_content(&mut file) {
            Ok(content) => content,
            Err(err) => {
                eprintln!("Log watcher read error: {err}");
                continue;
            }
        };

        if content.is_empty() {
            continue;
        }

        let Some(area_id) = extract_last_area_id(&content) else {
            continue;
        };

        logging::info("log_watcher", &format!("detected area: {area_id}"));

        let act_index_before = manager
            .get_current_progress()
            .ok()
            .map(|p| p.position.act_index);

        match manager.try_auto_advance(app, &area_id) {
            Ok(Some(page)) => {
                logging::info(
                    "log_watcher",
                    &format!(
                        "auto-advanced to act_index={} page_index={}",
                        page.position.act_index, page.position.page_index
                    ),
                );

                if let Ok(progress) = manager.get_current_progress() {
                    if let Err(err) = save_leveling_guide_progress(app, &progress) {
                        logging::error(
                            "log_watcher",
                            &format!("failed to persist progress: {:?}", err),
                        );
                    }
                }

                if let Some(prev_act) = act_index_before {
                    let new_act = page.position.act_index;
                    if new_act != prev_act {
                        logging::info(
                            "log_watcher",
                            &format!(
                                "act transition detected: {} -> {}, notifying timer with completed_act_index={}",
                                prev_act, new_act, prev_act
                            ),
                        );
                        if let Err(err) = timer_manager.notify_act_completed(app, prev_act) {
                            logging::error(
                                "log_watcher",
                                &format!("failed to notify timer of act completion: {:?}", err),
                            );
                        }
                    }
                }

                if let Err(err) = app.emit(LEVELING_GUIDE_PAGE_UPDATED_EVENT, &page) {
                    logging::error(
                        "log_watcher",
                        &format!("failed to emit page update: {err}"),
                    );
                }
            }
            Ok(None) => {}
            Err(err) => {
                logging::error(
                    "log_watcher",
                    &format!("auto-advance error for area {area_id}: {:?}", err),
                );
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_area_id_from_generating_level_line() {
        let line = r#"2024/01/15 10:30:45 12345 abc [INFO Client 1234] Generating level 6 area "1_1_5" with seed 42"#;
        assert_eq!(parse_area_id_from_line(line), Some("1_1_5"));
    }

    #[test]
    fn parse_area_id_returns_none_for_unrelated_line() {
        let line = "2024/01/15 10:30:45 some random log line";
        assert_eq!(parse_area_id_from_line(line), None);
    }

    #[test]
    fn extract_last_area_id_from_multiple_lines() {
        let content = r#"2024/01/15 10:30:45 12345 abc [INFO Client 1234] Generating level 6 area "1_1_5" with seed 42
some other line
2024/01/15 10:31:00 12345 abc [INFO Client 1234] Generating level 7 area "1_1_6" with seed 99
"#;
        assert_eq!(extract_last_area_id(content), Some("1_1_6".to_string()));
    }

    #[test]
    fn extract_last_area_id_returns_none_when_no_generating_lines() {
        let content = "line 1\nline 2\nline 3\n";
        assert_eq!(extract_last_area_id(content), None);
    }
}
