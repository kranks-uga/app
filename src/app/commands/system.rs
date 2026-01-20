//! Системные команды (выключение, перезагрузка)

use std::process::Command;
use crate::app::chat::DialogState;

/// Обработка системных команд (показывает диалог подтверждения)
pub fn process_system_command(cmd: &str, dialog: &mut DialogState) -> Option<String> {
    match cmd {
        "выключить пк" | "выключить компьютер" => {
            dialog.show_confirm(
                "Выключение компьютера",
                "Вы уверены, что хотите выключить компьютер?",
                "__shutdown__",
            );
            Some("Подтвердите выключение...".into())
        }
        "перезагрузить" | "рестарт" => {
            dialog.show_confirm(
                "Перезагрузка",
                "Вы уверены, что хотите перезагрузить компьютер?",
                "__reboot__",
            );
            Some("Подтвердите перезагрузку...".into())
        }
        _ => None,
    }
}

/// Выполнить выключение (вызывается после подтверждения)
pub fn execute_shutdown() -> String {
    let (prog, args) = if cfg!(windows) {
        ("shutdown", vec!["/s", "/t", "0"])
    } else {
        ("shutdown", vec!["-h", "now"])
    };

    match Command::new(prog).args(&args).status() {
        Ok(_) => "Выключение...".into(),
        Err(e) => format!("Ошибка: {}", e),
    }
}

/// Выполнить перезагрузку (вызывается после подтверждения)
pub fn execute_reboot() -> String {
    let (prog, args) = if cfg!(windows) {
        ("shutdown", vec!["/r", "/t", "0"])
    } else {
        ("shutdown", vec!["-r", "now"])
    };

    match Command::new(prog).args(&args).status() {
        Ok(_) => "Перезагрузка...".into(),
        Err(e) => format!("Ошибка: {}", e),
    }
}
