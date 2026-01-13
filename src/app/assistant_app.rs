use super::config::Config;
use super::chat::{ChatHistory, TaskManager, DialogType};
use super::commands;
use super::ui;
use eframe::egui;
use std::sync::mpsc;

/// Центральное хранилище состояния приложения (Single Source of Truth)
pub struct AssistantApp {
    pub config: Config,           // Настройки (имя, цвета)
    pub chat_history: ChatHistory, // История сообщений
    pub task_manager: TaskManager, // Модуль фоновых потоков
    
    pub input_text: String,       // Буфер текущего ввода пользователя
    pub show_settings: bool,      // Флаг видимости боковой панели
    
    // Состояние модальных окон
    pub show_dialog: bool,
    pub dialog_type: DialogType,
    pub dialog_title: String,
    pub dialog_message: String,
    pub dialog_input: String,
    pub dialog_package: String,
    
    // Канал получения ответов от асинхронных задач (MPSC)
    pub task_result_receiver: mpsc::Receiver<String>,
}

impl AssistantApp {
    /// Инициализация приложения, менеджера задач и стартового приветствия
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let (task_manager, task_result_receiver) = TaskManager::new();
        
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
    
    /// Логика обработки ввода: добавление в чат и вызов парсера команд
    pub fn process_input(&mut self) {
        if self.input_text.trim().is_empty() {
            return;
        }
        
        let input = self.input_text.clone();
        self.chat_history.add_message("Вы".to_string(), input.clone());
        
        // Передача состояния в обработчик команд для принятия решений (UI или системных)
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
        
        self.input_text.clear(); // Очистка поля после обработки
    }
    
    /// Неблокирующая проверка завершенных фоновых задач (пакеты, системные чеки)
    pub fn check_background_tasks(&mut self) {
        while let Ok(result) = self.task_result_receiver.try_recv() {
            self.chat_history.add_message("Система".to_string(), result);
        }
    }
}

// Главный цикл отрисовки (обновляется при каждом событии/кадре)
impl eframe::App for AssistantApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Опрос фоновых потоков в начале каждого кадра
        self.check_background_tasks();
        
        // Глобальная стилизация виджетов (отступы и скругления)
        let mut style = (*ctx.style()).clone();
        style.spacing.item_spacing = egui::vec2(12.0, 12.0);
        style.visuals.widgets.inactive.rounding = 12.0.into();
        ctx.set_style(style);
        
        // Делегирование рендеринга модулю UI
        ui::render_ui(ctx, self);
    }
}
