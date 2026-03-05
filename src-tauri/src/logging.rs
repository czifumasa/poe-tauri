use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::SystemTime;

static LOG_FILE: OnceLock<PathBuf> = OnceLock::new();

const LOG_FILE_NAME: &str = if cfg!(debug_assertions) {
    "poe-tauri.dev.log"
} else {
    "poe-tauri.log"
};

pub fn init(app: &tauri::AppHandle) {
    let dir = resolve_log_dir(app);

    if let Err(err) = fs::create_dir_all(&dir) {
        eprintln!("Failed to create log directory {}: {err}", dir.display());
        return;
    }

    let path = dir.join(LOG_FILE_NAME);

    if let Err(err) = fs::write(&path, "") {
        eprintln!("Failed to clear log file {}: {err}", path.display());
        return;
    }

    if LOG_FILE.set(path.clone()).is_err() {
        return;
    }

    info("logging", &format!("Logging initialized at {}", path.display()));
}

pub fn info(tag: &str, message: &str) {
    write_entry("INFO", tag, message);
}

pub fn warn(tag: &str, message: &str) {
    write_entry("WARN", tag, message);
}

pub fn error(tag: &str, message: &str) {
    write_entry("ERROR", tag, message);
}

fn resolve_log_dir(app: &tauri::AppHandle) -> PathBuf {
    use tauri::Manager;

    app.path()
        .app_log_dir()
        .unwrap_or_else(|_| {
            app.path()
                .app_data_dir()
                .map(|p| p.join("logs"))
                .unwrap_or_else(|_| PathBuf::from("logs"))
        })
}

fn format_timestamp() -> String {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs();
    let millis = now.subsec_millis();

    let remaining_day = secs % 86400;
    let hours = remaining_day / 3600;
    let minutes = (remaining_day % 3600) / 60;
    let seconds = remaining_day % 60;

    format!("{:02}:{:02}:{:02}.{:03}", hours, minutes, seconds, millis)
}

fn write_entry(level: &str, tag: &str, message: &str) {
    let Some(path) = LOG_FILE.get() else {
        return;
    };

    let timestamp = format_timestamp();
    let line = format!("[{timestamp}] [{level}] [{tag}] {message}\n");

    let result = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .and_then(|mut file| file.write_all(line.as_bytes()));

    if let Err(err) = result {
        eprintln!("Failed to write log entry: {err}");
    }
}
