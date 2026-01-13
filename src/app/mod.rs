
// src/app/mod.rs

// Регистрация внутренних компонентов системы в иерархии модулей
pub mod assistant_app; // Ядро приложения и управление состоянием
pub mod config;        // Настройки и параметры оформления
pub mod chat;          // Структуры данных сообщений и менеджер фоновых задач
pub mod commands;      // Логика разбора и выполнения команд
pub mod ui;            // Отрисовка интерфейса и визуальных компонентов

// Удобный доступ: позволяет использовать app::AssistantApp напрямую вместо app::assistant_app::AssistantApp
pub use assistant_app::AssistantApp;
