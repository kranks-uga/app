mod app; // Подключение корневого модуля приложения

use app::AssistantApp;
use eframe::egui;

/// Точка входа в программу. Используем Tokio для асинхронности ИИ.
#[tokio::main]
async fn main() -> Result<(), eframe::Error> { // <-- Делаем функцию асинхронной
    // Настройка параметров графического окна
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 750.0]) // Начальное разрешение экрана
            .with_title("Alfons Assistant"),    // Заголовок окна в ОС
        ..Default::default()
    };
    
    // Запуск нативного приложения (инициализация графического бэкенда)
    eframe::run_native(
        "Alfons AI", // Уникальный ID приложения
        options,
        // Передача контекста eframe в конструктор ассистента
        Box::new(|cc| Box::new(AssistantApp::new(cc))),
    )
}