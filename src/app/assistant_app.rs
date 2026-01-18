use super::config::Config;
use super::chat::{ChatHistory, TaskManager, DialogType};
use super::commands;
use super::ui;
use super::ai::local_provider::LocalAi; // Импорт вашего нового модуля ИИ
use eframe::egui;
use std::sync::{mpsc, Arc};

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

    // Движок локального ИИ (обернут в Arc для безопасной передачи между потоками)
    pub ai: Arc<LocalAi>,
}

impl AssistantApp {
    /// Инициализация приложения, менеджера задач и стартового приветствия
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let (task_manager, task_result_receiver) = TaskManager::new();
        
        let config = Config::load(); 

        let mut chat_history = ChatHistory::new(100);
        chat_history.add_message(
            config.assistant_name.clone(),
            "Система Arch Linux готова. Введите команду или задайте вопрос ИИ.".to_string(),
        );

        // Инициализируем локальный ИИ (убедитесь, что модель llama3 скачана в ollama)
        let ai = Arc::new(LocalAi::new("llama3"));
        
        Self {
            config,
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
            ai,
        }
    }
    
    /// Логика обработки ввода: добавление в чат и вызов парсера команд или ИИ
    pub fn process_input(&mut self) {
        let trimmed_input = self.input_text.trim();
        if trimmed_input.is_empty() {
            return;
        }
        
        let input = trimmed_input.to_string();
        self.chat_history.add_message("Вы".to_string(), input.clone());
        
        // 1. Пробуем обработать как жесткую системную команду (быстрые действия)
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
            // Если команда найдена (например, !help или !exit), выводим её результат сразу
            self.chat_history.add_message(
                self.config.assistant_name.clone(),
                response_text,
            );
        } else {
            // 2. Если это не команда — отправляем запрос в локальный ИИ асинхронно
            let ai_engine = Arc::clone(&self.ai);
            let tx = self.task_manager.result_sender.clone();  // Используем sender вашего TaskManager
            let assistant_name = self.config.assistant_name.clone();

            // Запускаем асинхронную задачу, чтобы UI не зависал во время "раздумий" ИИ
            tokio::spawn(async move {
                match ai_engine.generate(&input).await {
                    Ok(ai_response) => {
                        // Отправляем ответ обратно в основной поток через канал
                        let _ = tx.send(format!("{}: {}", assistant_name, ai_response));
                    }
                    Err(e) => {
                        let _ = tx.send(format!("Ошибка ИИ: {}", e));
                    }
                }
            });
        }
        
        self.input_text.clear(); // Очистка поля после обработки
    }
    
    /// Неблокирующая проверка завершенных фоновых задач (пакеты, ИИ, системные чеки)
    pub fn check_background_tasks(&mut self) {
        // Здесь мы принимаем сообщения как от системных задач, так и от ИИ
        while let Ok(result) = self.task_result_receiver.try_recv() {
            // Если в сообщении есть имя ассистента (из tokio::spawn), выводим как ответ ассистента
            if result.starts_with(&self.config.assistant_name) {
                // Разделяем имя и текст для красоты
                let parts: Vec<&str> = result.splitn(2, ": ").collect();
                if parts.len() == 2 {
                    self.chat_history.add_message(parts[0].to_string(), parts[1].to_string());
                } else {
                    self.chat_history.add_message("Система".to_string(), result);
                }
            } else {
                self.chat_history.add_message("Система".to_string(), result);
            }
        }
    }

    pub fn clear_chat(&mut self) {
        self.chat_history.clear();
        self.chat_history.add_message(
            self.config.assistant_name.clone(),
            "История чата очищена. Чем могу помочь?".to_string(),
        );
    }
}

// Главный цикл отрисовки (обновляется при каждом событии/кадре)
impl eframe::App for AssistantApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Опрос фоновых потоков (включая ответы ИИ) в начале каждого кадра
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
