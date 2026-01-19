//! Системные команды (выключение, перезагрузка)

use std::process::Command;

/// Обработка системных команд
pub fn process_system_command(cmd: &str) -> Option<String> {
    let (prog, args) = if cfg!(windows) {
        match cmd {
            "выключить пк" | "выключить компьютер" => ("shutdown", vec!["/s", "/t", "0"]),
            "перезагрузить" | "рестарт" => ("shutdown", vec!["/r", "/t", "0"]),
            _ => return None,
        }
    } else {
        match cmd {
            "выключить пк" | "выключить компьютер" => ("shutdown", vec!["-h", "now"]),
            "перезагрузить" | "рестарт" => ("shutdown", vec!["-r", "now"]),
            _ => return None,
        }
    };

    match Command::new(prog).args(&args).status() {
        Ok(_) => Some("Команда отправлена.".into()),
        Err(e) => Some(format!("Ошибка: {}", e)),
    }
}
