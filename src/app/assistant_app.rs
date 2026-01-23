//! Главная структура приложения

use super::ai::local_provider::LocalAi;
use super::chat::{ChatHistory, DialogState, InputHistory, TaskManager};
use super::commands::{self, base::CMD_CLEAR_CHAT};
use super::config::Config;
use super::constants::messages;
use super::desktop::{DeStyles, DesktopEnvironment};
use super::guides::GuideRegistry;
use super::ui;
use eframe::egui;
use regex::Regex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, OnceLock};
use std::time::{Duration, Instant};

/// Интервал проверки статуса Ollama (в секундах)
const OLLAMA_CHECK_INTERVAL: u64 = 30;

/// Статический Regex для парсинга [CMD:...] маркеров
fn cmd_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\[CMD:([^\]]+)\]").expect("Invalid CMD regex"))
}

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
    pub input_history: InputHistory,
    pub ollama_online: Arc<AtomicBool>,
    pub ollama_installed: Arc<AtomicBool>,
    pub yay_installed: Arc<AtomicBool>,
    pub custom_model_exists: Arc<AtomicBool>,
    pub app_installed: Arc<AtomicBool>,
    last_ollama_check: Instant,

    // Окружение рабочего стола
    pub desktop_env: DesktopEnvironment,
    pub de_styles: DeStyles,

    // Фоновые задачи
    pub tasks: TaskManager,
    task_receiver: mpsc::Receiver<String>,
}

impl AssistantApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let (tasks, task_receiver) = TaskManager::new();
        let config = Config::load();

        // Определяем окружение рабочего стола
        let desktop_env = DesktopEnvironment::detect();
        let de_styles = DeStyles::for_de(desktop_env);

        let mut chat = ChatHistory::default();
        chat.add_message(&config.assistant_name, messages::WELCOME);

        let ai = Arc::new(LocalAi::new());
        ai.set_model(&config.ollama_model);

        // Запускаем проверку статуса Ollama в фоне
        let ollama_online = Arc::new(AtomicBool::new(false));
        let ollama_online_clone = ollama_online.clone();
        tokio::spawn(async move {
            let status = super::ai::local_provider::check_ollama_status().await;
            ollama_online_clone.store(status, Ordering::SeqCst);
        });

        // Проверяем, установлена ли Ollama
        let ollama_installed = Arc::new(AtomicBool::new(false));
        let ollama_installed_clone = ollama_installed.clone();
        std::thread::spawn(move || {
            let status = super::ai::local_provider::is_ollama_installed();
            ollama_installed_clone.store(status, Ordering::SeqCst);
        });

        // Запускаем проверку yay в фоне (чтобы не блокировать UI)
        let yay_installed = Arc::new(AtomicBool::new(false));
        let yay_installed_clone = yay_installed.clone();
        std::thread::spawn(move || {
            let status = super::commands::package::is_yay_installed();
            yay_installed_clone.store(status, Ordering::SeqCst);
        });

        // Проверяем существование кастомной модели
        let custom_model_exists = Arc::new(AtomicBool::new(false));
        let custom_model_clone = custom_model_exists.clone();
        std::thread::spawn(move || {
            let exists = super::ai::local_provider::is_custom_model_exists();
            custom_model_clone.store(exists, Ordering::SeqCst);
        });

        // Проверяем, установлено ли приложение в систему
        let app_installed = Arc::new(AtomicBool::new(super::installer::is_installed()));

        Self {
            config,
            chat,
            guides: GuideRegistry::new(),
            ai,
            input_text: String::new(),
            show_settings: false,
            dialog: DialogState::new(),
            input_history: InputHistory::new(),
            ollama_online,
            ollama_installed,
            yay_installed,
            custom_model_exists,
            app_installed,
            last_ollama_check: Instant::now(),
            desktop_env,
            de_styles,
            tasks,
            task_receiver,
        }
    }

    /// Периодическая проверка статуса Ollama
    fn check_ollama_periodic(&mut self) {
        if self.last_ollama_check.elapsed() >= Duration::from_secs(OLLAMA_CHECK_INTERVAL) {
            self.last_ollama_check = Instant::now();
            let ollama_online = self.ollama_online.clone();
            tokio::spawn(async move {
                let status = super::ai::local_provider::check_ollama_status().await;
                ollama_online.store(status, Ordering::SeqCst);
            });
        }
    }

    /// Обработка ввода пользователя
    pub fn process_input(&mut self) {
        let input = self.input_text.trim();
        if input.is_empty() {
            return;
        }

        let input = input.to_string();
        self.input_history.push(&input);
        self.chat.add_message("Вы", &input);

        // Пробуем обработать как команду
        let response = commands::process_command(
            &input,
            &self.config.assistant_name,
            &mut self.dialog,
            &self.tasks,
            &self.guides,
        );

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
                    // Обрабатываем команды от AI
                    let processed_text = self.process_ai_commands(text);
                    self.chat.add_message(name, &processed_text);
                } else {
                    self.chat.add_message("Система", &result);
                }
            } else {
                self.chat.add_message("Система", &result);
            }
        }
    }

    /// Обрабатывает маркеры [CMD:...] в ответе AI и выполняет команды
    fn process_ai_commands(&mut self, text: &str) -> String {
        let cmd_re = cmd_regex();
        let mut result = text.to_string();

        // Находим все команды в тексте
        let commands: Vec<String> = cmd_re
            .captures_iter(text)
            .map(|cap| cap[1].to_string())
            .collect();

        // Выполняем каждую команду
        for cmd in commands {
            let marker = format!("[CMD:{}]", cmd);

            let cmd_response = commands::process_command(
                &cmd,
                &self.config.assistant_name,
                &mut self.dialog,
                &self.tasks,
                &self.guides,
            );

            if let Some(response) = cmd_response {
                // Проверяем специальные команды
                if response == commands::base::CMD_CLEAR_CHAT {
                    self.clear_chat();
                    result = result.replace(&marker, "");
                } else {
                    // Убираем маркер, оставляем только текст AI
                    // Результат команды будет показан через диалог или системное сообщение
                    result = result.replace(&marker, "");
                }
            } else {
                // Команда не распознана - показываем ошибку
                result = result.replace(&marker, &format!("[!] команда '{}' не распознана", cmd));
            }
        }

        result
    }

    /// Очистка чата
    pub fn clear_chat(&mut self) {
        self.chat.clear();
        self.chat
            .add_message(&self.config.assistant_name, messages::CHAT_CLEARED);
    }
}

impl eframe::App for AssistantApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.check_tasks();
        self.check_ollama_periodic();

        // Стили адаптированные под DE
        let mut style = (*ctx.style()).clone();
        style.spacing.item_spacing = egui::vec2(self.de_styles.spacing, self.de_styles.spacing);
        style.visuals.widgets.inactive.rounding = self.de_styles.rounding.into();
        style.visuals.widgets.active.rounding = self.de_styles.rounding.into();
        style.visuals.widgets.hovered.rounding = self.de_styles.rounding.into();
        style.visuals.widgets.noninteractive.rounding = self.de_styles.rounding.into();
        style.visuals.window_rounding = self.de_styles.rounding.into();
        ctx.set_style(style);

        ui::render(ctx, self);
    }
}
