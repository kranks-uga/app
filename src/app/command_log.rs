//! Логирование выполненных команд

use chrono::Local;
use std::fs::{OpenOptions, create_dir_all};
use std::io::Write;
use std::path::PathBuf;

/// Записывает команду в лог-файл
pub fn log_command(command: &str, result: &str) {
    if let Some(log_path) = get_log_path() {
        // Создаём директорию если нужно
        if let Some(parent) = log_path.parent() {
            let _ = create_dir_all(parent);
        }

        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
        {
            let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
            let _ = writeln!(file, "[{}] CMD: {} -> {}", timestamp, command, result);
        }
    }
}

/// Возвращает путь к файлу лога
fn get_log_path() -> Option<PathBuf> {
    dirs::data_local_dir().map(|p| p.join("alfons-assistant").join("commands.log"))
}

/// Читает последние N записей из лога
pub fn read_last_entries(count: usize) -> Vec<String> {
    get_log_path()
        .and_then(|path| std::fs::read_to_string(path).ok())
        .map(|content| {
            content
                .lines()
                .rev()
                .take(count)
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect()
        })
        .unwrap_or_default()
}
