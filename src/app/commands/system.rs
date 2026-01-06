use std::process::Command;

/// Обрабатывает системные команды (выключение, перезагрузка)
pub fn process_system_command(cmd: &str) -> Option<String> {
    match cmd {
        "выключить пк" | "выключить компьютер" => {
            match Command::new("shutdown").args(&["-h", "now"]).status() {
                Ok(_) => Some("Система выключается...".to_string()),
                Err(e) => Some(format!("Ошибка при выключении: {}", e)),
            }
        }
        "перезагрузить" | "рестарт" => {
            match Command::new("shutdown").args(&["-r", "now"]).status() {
                Ok(_) => Some("Система перезагружается...".to_string()),
                Err(e) => Some(format!("Ошибка при перезагрузке: {}", e)),
            }
        }
        _ => None,
    }
}

/// Выполняет произвольную shell-команду
pub fn execute_shell_command(shell_cmd: &str) -> String {
    if shell_cmd.is_empty() {
        return "Какую команду выполнить?".to_string();
    }
    
    match Command::new("sh").args(&["-c", shell_cmd]).output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            
            if !stdout.is_empty() {
                format!("Результат:\n{}", stdout.trim())
            } else if !stderr.is_empty() {
                format!("Ошибка:\n{}", stderr.trim())
            } else {
                "Команда выполнена (нет вывода)".to_string()
            }
        }
        Err(e) => format!("Ошибка выполнения: {}", e),
    }
}