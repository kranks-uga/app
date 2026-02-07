use chrono::Local;
use crate::app::constants::APP_VERSION;

/// Специальные команды для перехвата в UI
pub const CMD_CLEAR_CHAT: &str = "COMMAND_ACTION_CLEAR";

/// Обработка базовых текстовых команд (приветствие, время, дата, помощь)
pub fn process_basic_command(cmd: &str, assistant_name: &str) -> Option<String> {
    match cmd {
        // Приветствие
        "привет" | "здравствуй" | "хай" | "hello" => Some(format!(
            "Привет! Я {}, твой помощник для Arch Linux.",
            assistant_name
        )),

        // Версия
        "версия" => {
            Some(format!("Нынешняя версия проекта: {}",APP_VERSION))
        }

        // Очистка чата — возвращаем маркер для перехвата в UI
        "очистить" | "очистить чат" | "clear" => {
            Some(CMD_CLEAR_CHAT.to_string())
        }

        // Повторение фразы
        cmd if cmd.starts_with("скажи ") => {
            let message = cmd.trim_start_matches("скажи ").trim();
            if message.is_empty() {
                Some("Что именно сказать?".to_string())
            } else {
                Some(message.to_string())
            }
        }

        // Время
        "время" | "который час" | "time" => Some(format!(
            "Текущее время: {}",
            Local::now().format("%H:%M:%S")
        )),

        // Дата
        "дата" | "какое сегодня число" | "date" => {
            Some(format!("Сегодня: {}", Local::now().format("%d.%m.%Y")))
        }

        // Дата и время
        "дата и время" => Some(format!(
            "Сейчас: {}",
            Local::now().format("%d.%m.%Y %H:%M:%S")
        )),

        // Помощь
        "помощь" | "help" | "?" => Some(HELP_TEXT.to_string()),

        _ => None,
    }
}

/// Текст справки
const HELP_TEXT: &str = "\
📋 Доступные команды:

▸ Базовые:
  время, дата, дата и время

▸ Пакеты (через yay):
  поиск <запрос>
  установить <пакет>
  удалить <пакет>
  обновить систему

▸ Система:
  выключить пк
  перезагрузить

▸ Гайды:
  гайды — список всех гайдов
  гайд <тема> — показать гайд

▸ Прочее:
  очистить — очистить чат
  помощь — эта справка

💡 Или просто задайте вопрос — ИИ постарается помочь!";
