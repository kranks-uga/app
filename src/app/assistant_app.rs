use super::config::Config;
use super::chat::{ChatHistory, TaskManager, DialogType};
use super::commands;
use super::ui;
use eframe::egui;
use std::sync::mpsc;

/// Главная структура приложения - хранит все состояние
pub struct AssistantApp {
    // Основные компоненты
    pub config: Config,
    pub chat_history: ChatHistory,
    pub task_manager: TaskManager,
    
    // Пользовательский ввод
    pub input_text: String,
    
    // UI состояние
    pub show_settings: bool,
    
    // Диалоги
    pub show_dialog: bool,
    pub dialog_type: DialogType,
    pub dialog_title: String,
    pub dialog_message: String,
    pub dialog_input: String,
    pub dialog_package: String,
    
    // Канал для получения результатов фоновых задач
    pub task_result_receiver: mpsc::Receiver<String>,
}

impl AssistantApp {
    /// Создает новый экземпляр приложения
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Инициализируем менеджер задач и канал для результатов
        let (task_manager, task_result_receiver) = TaskManager::new();
        
        // Создаем историю чата
        let mut chat_history = ChatHistory::new(100);
        chat_history.add_message(
            "Альфонс".to_string(),
            "Система Arch Linux готова. Введите команду или откройте настройки.".to_string(),
        );
        
        Self {
            config: Config::new(),
            chat_history,
            task_manager,
            input_text: String::new(),
            show_settings: false,
            show_dialog: false,
            dialog_type: DialogType::Info,
            dialog_title: String::new(),
            dialog_message: String::new(),
            dialog_input: String::new(),
            dialog_package: String::new(),
            task_result_receiver,
        }
    }
    
    /// Обрабатывает пользовательский ввод
    pub fn process_input(&mut self) {
        if self.input_text.trim().is_empty() {
            return;
        }
        
        let input = self.input_text.clone();
        self.chat_history.add_message("Вы".to_string(), input.clone());
        
        // Обрабатываем команду
        let response = commands::process_command(
            &input,
            &self.config.assistant_name,
            &mut self.dialog_type,
            &mut self.dialog_title,
            &mut self.dialog_message,
            &mut self.dialog_input,
            &mut self.dialog_package,
            &mut self.show_dialog,
            &self.task_manager,
        );
        
        if let Some(response_text) = response {
            self.chat_history.add_message(
                self.config.assistant_name.clone(),
                response_text,
            );
        }
        
        self.input_text.clear();
    }
    
    /// Проверяет результаты фоновых задач
    pub fn check_background_tasks(&mut self) {
        // Пытаемся получить результат без блокировки
        while let Ok(result) = self.task_result_receiver.try_recv() {
            self.chat_history.add_message("Система".to_string(), result);
        }
    }
}

// Реализация интерфейса eframe::App
impl eframe::App for AssistantApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Проверяем результаты фоновых задач
        self.check_background_tasks();
        
        // Настраиваем стиль интерфейса
        let mut style = (*ctx.style()).clone();
        style.spacing.item_spacing = egui::vec2(12.0, 12.0);
        style.visuals.widgets.inactive.rounding = 12.0.into();
        ctx.set_style(style);
        
        // Отрисовываем весь интерфейс
        ui::render_ui(ctx, self);
    }
}