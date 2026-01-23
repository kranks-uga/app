//! Главный модуль приложения Альфонс
//!
//! Структура:
//! - `constants` — все константы и строки
//! - `config` — настройки пользователя
//! - `chat` — история сообщений и фоновые задачи
//! - `commands` — обработка команд
//! - `guides` — обучающие гайды
//! - `ai` — интеграция с Ollama
//! - `ui` — графический интерфейс
//! - `assistant_app` — главная структура приложения
//! - `installer` — установка в систему

pub mod ai; // Локальный ИИ (Ollama)
pub mod assistant_app; // Главная структура
pub mod chat; // История и фоновые задачи
pub mod command_log; // Логирование команд
pub mod commands; // Обработка команд
pub mod config; // Настройки пользователя
pub mod constants; // Константы и строки
pub mod desktop;
pub mod guides; // Обучающие гайды
pub mod installer; // Установка в систему
pub mod ui; // Графический интерфейс // Определение DE и стили

pub use assistant_app::AssistantApp;
