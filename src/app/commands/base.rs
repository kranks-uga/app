use chrono::Local;

/// Обрабатывает базовые команды (приветствие, время, дата)
pub fn process_basic_command(cmd: &str, assistant_name: &str) -> Option<String> {
    match cmd {
        "привет" | "здравствуй" => {
            Some(format!("Привет! Я {}, твой ассистент на Arch Linux.", assistant_name))
        }
        "очистить" | "очистить чат" => {
            // Эта команда требует специальной обработки в основном приложении
            None
        }
        cmd if cmd.starts_with("скажи ") => {
            let message = cmd.trim_start_matches("скажи ").trim();
            if message.is_empty() {
                Some("Что именно сказать?".to_string())
            } else {
                Some(message.to_string())
            }
        }
        "время" | "который час" => {
            let now = Local::now();
            Some(format!("Текущее время: {}", now.format("%H:%M:%S")))
        }
        "дата" | "какое сегодня число" => {
            let now = Local::now();
            Some(format!("Сегодня: {}", now.format("%d.%m.%Y")))
        }
        "дата и время" => {
            let now = Local::now();
            Some(format!("Текущие дата и время: {}", now.format("%d.%m.%Y %H:%M:%S")))
        }
        "помощь" => format!("помощь").into(),
        _ => None,
    }
}