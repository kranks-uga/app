//! Логирование выполненных команд

use chrono::Local;
use std::fs::{create_dir_all, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

/// Записывает команду в лог-файл
pub fn log_command(command: &str, result: &str) {
    if let Some(log_path) = get_log_path() {
        // Создаём директорию если нужно
        if let Some(parent) = log_path.parent() {
            let _ = create_dir_all(parent);
        }

        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&log_path) {
            let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
            let _ = writeln!(file, "[{}] CMD: {} -> {}", timestamp, command, result);
        }
    }
}

/// Возвращает путь к файлу лога
fn get_log_path() -> Option<PathBuf> {
    dirs::data_local_dir().map(|p| p.join("alfons-assistant").join("commands.log"))
}
