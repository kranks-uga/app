mod app; // Подключение корневого модуля приложения

use app::AssistantApp;
use eframe::egui;

/// Точка входа в программу. Используем Tokio для асинхронности ИИ.
#[tokio::main]
async fn main() -> Result<(), eframe::Error> {
    // <-- Делаем функцию асинхронной
    // Настройка параметров графического окна
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 750.0])
            .with_min_inner_size([640.0, 480.0])
            .with_title("Gаврик Assistant"),
        ..Default::default()
    };

    // Запуск нативного приложения (инициализация графического бэкенда)
    eframe::run_native(
        "Gаврик AI", // Уникальный ID приложения
        options,
        // Передача контекста eframe в конструктор ассистента
        Box::new(|cc| Box::new(AssistantApp::new(cc))),
    )
}
