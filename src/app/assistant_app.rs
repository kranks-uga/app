//! Главная структура приложения

use super::config::Config;
use super::chat::{ChatHistory, TaskManager, DialogState};
use super::commands::{self, base::CMD_CLEAR_CHAT};
use super::constants::messages;
use super::guides::GuideRegistry;
use super::ui;
use super::ai::local_provider::LocalAi;
use eframe::egui;
use std::sync::{mpsc, Arc};

/// Центральное хранилище состояния приложения
pub struct AssistantApp {
    // Данные
    pub config: Config,
    pub chat: ChatHistory,
    pub guides: GuideRegistry,
    pub ai: Arc<LocalAi>,

    // UI состояние
    pub input_text: String,
    pub show_settings: bool,
    pub dialog: DialogState,

    // Фоновые задачи
    pub tasks: TaskManager,
    task_receiver: mpsc::Receiver<String>,
}

impl AssistantApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let (tasks, task_receiver) = TaskManager::new();
        let config = Config::load();

        let mut chat = ChatHistory::default();
        chat.add_message(&config.assistant_name, messages::WELCOME);

        Self {
            config,
            chat,
            guides: GuideRegistry::new(),
            ai: Arc::new(LocalAi::new()),
            input_text: String::new(),
            show_settings: false,
            dialog: DialogState::new(),
            tasks,
            task_receiver,
        }
    }

    /// Обработка ввода пользователя
    pub fn process_input(&mut self) {
        let input = self.input_text.trim();
        if input.is_empty() {
            return;
        }

        let input = input.to_string();
        self.chat.add_message("Вы", &input);

        // Пробуем обработать как команду
        let response = commands::process_command(&input, &self.config.assistant_name, &mut self.dialog, &self.tasks, &self.guides);

        if let Some(text) = response {
            // Проверяем специальные команды
            if text == CMD_CLEAR_CHAT {
                self.clear_chat();
            } else {
                self.chat.add_message(&self.config.assistant_name, text);
            }
        } else {
            // Отправляем в AI
            self.send_to_ai(&input);
        }

        self.input_text.clear();
    }

    /// Отправка запроса в AI
    fn send_to_ai(&self, input: &str) {
        let ai = Arc::clone(&self.ai);
        let tx = self.tasks.result_sender.clone();
        let name = self.config.assistant_name.clone();
        let input = input.to_string();

        tokio::spawn(async move {
            let response = match ai.generate(&input).await {
                Ok(text) => format!("{}: {}", name, text),
                Err(e) => format!("Ошибка ИИ: {}", e),
            };
            let _ = tx.send(response);
        });
    }

    /// Проверка завершённых фоновых задач
    pub fn check_tasks(&mut self) {
        while let Ok(result) = self.task_receiver.try_recv() {
            // AI ответы содержат имя ассистента
            if result.starts_with(&self.config.assistant_name) {
                if let Some((name, text)) = result.split_once(": ") {
                    self.chat.add_message(name, text);
                } else {
                    self.chat.add_message("Система", &result);
                }
            } else {
                self.chat.add_message("Система", &result);
            }
        }
    }

    /// Очистка чата
    pub fn clear_chat(&mut self) {
        self.chat.clear();
        self.chat.add_message(&self.config.assistant_name, messages::CHAT_CLEARED);
    }
}

impl eframe::App for AssistantApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.check_tasks();

        // Стили
        let mut style = (*ctx.style()).clone();
        style.spacing.item_spacing = egui::vec2(12.0, 12.0);
        style.visuals.widgets.inactive.rounding = 12.0.into();
        ctx.set_style(style);

        ui::render(ctx, self);
    }
}
