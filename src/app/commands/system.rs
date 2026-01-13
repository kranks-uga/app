use std::process::Command;

/// Обработка системных команд выключения и перезагрузки
pub fn process_system_command(cmd: &str) -> Option<String> {
    // Определяем параметры команды в зависимости от ОС
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

    // Выполнение команды и обработка результата
    match Command::new(prog).args(&args).status() {
        Ok(_) => Some("Команда отправлена успешно".to_string()),
        Err(e) => Some(format!("Ошибка выполнения: {}", e)),
    }
}

/// Выполнение произвольной команды через системную оболочку (sh или cmd)
pub fn execute_shell_command(shell_cmd: &str) -> String {
    if shell_cmd.is_empty() {
        return "Команда не указана".to_string();
    }

    // Выбор оболочки: cmd для Windows, sh для остальных ОС
    let (shell, flag) = if cfg!(windows) { ("cmd", "/C") } else { ("sh", "-c") };

    match Command::new(shell).args(&[flag, shell_cmd]).output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            if !stdout.is_empty() {
                format!("Вывод:\n{}", stdout.trim())
            } else if !stderr.is_empty() {
                format!("Ошибка:\n{}", stderr.trim())
            } else {
                "Выполнено (пустой вывод)".to_string()
            }
        }
        Err(e) => format!("Не удалось запустить оболочку: {}", e),
    }
}
