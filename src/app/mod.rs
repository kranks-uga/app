// Экспорт модулей приложения
pub mod assistant_app;
pub mod config;
pub mod chat;
pub mod commands;
pub mod ui;

// Реэкспорт главной структуры приложения
pub use assistant_app::AssistantApp;