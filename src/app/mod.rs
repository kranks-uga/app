//! Главный модуль приложения Gаврик
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
pub mod game; // Мини-игры
pub mod assistant_app; // Главная структура
pub mod chat; // История и фоновые задачи
pub mod command_log; // Логирование команд
pub mod commands; // Обработка команд
pub mod config; // Настройки пользователя
pub mod constants; // Константы и строки
pub mod desktop; // Определение DE и стили
pub mod guides; // Обучающие гайды
pub mod installer; // Установка в систему
pub mod ui; // Графический интерфейс

pub use assistant_app::AssistantApp;
