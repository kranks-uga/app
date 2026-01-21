//! Модуль чата и фоновых задач

use std::collections::VecDeque;
use std::sync::mpsc::{self, Sender, Receiver};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use chrono::{Local, DateTime};
use super::constants::MAX_CHAT_MESSAGES;

// ============================================================================
// Диалоги
// ============================================================================

/// Типы диалоговых окон
#[derive(Debug, Clone, Default, PartialEq)]
pub enum DialogType {
    #[default]
    Info,
    PackageSearch,
    Confirmation,
}

/// Состояние диалогового окна (упрощает передачу параметров)
#[derive(Debug, Clone, Default)]
pub struct DialogState {
    pub visible: bool,
    pub dialog_type: DialogType,
    pub title: String,
    pub message: String,
    pub input: String,
    pub package: String,
}

impl DialogState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Показать диалог поиска пакетов
    pub fn show_search(&mut self) {
        self.visible = true;
        self.dialog_type = DialogType::PackageSearch;
        self.title = "Поиск пакетов".to_string();
        self.message = "Введите название пакета:".to_string();
        self.input.clear();
    }

    /// Показать диалог подтверждения
    pub fn show_confirm(&mut self, title: &str, message: &str, package: &str) {
        self.visible = true;
        self.dialog_type = DialogType::Confirmation;
        self.title = title.to_string();
        self.message = message.to_string();
        self.package = package.to_string();
    }

    /// Скрыть диалог
    pub fn hide(&mut self) {
        self.visible = false;
        self.input.clear();
        self.package.clear();
    }
}

// ============================================================================
// Фоновые задачи
// ============================================================================

/// Типы фоновых задач
#[derive(Debug)]
pub enum BackgroundTask {
    SearchPackages(String),
    InstallPackage(String),
    RemovePackage(String),
    UpdateSystem,
    CheckYay,
    InstallYay,
    ShutdownSystem,
    RebootSystem,
    CreateCustomModel,
    InstallToSystem,
    UninstallFromSystem,
}

// ============================================================================
// История чата
// ============================================================================

/// Сообщение в чате
#[derive(Clone)]
pub struct ChatMessage {
    pub sender: String,
    pub text: String,
    pub timestamp: DateTime<Local>,
}

/// Управление историей чата
pub struct ChatHistory {
    messages: VecDeque<ChatMessage>,
    max_messages: usize,
}

impl ChatHistory {
    pub fn new(max_messages: usize) -> Self {
        Self {
            messages: VecDeque::with_capacity(max_messages),
            max_messages,
        }
    }

    /// Добавляет сообщение в историю
    pub fn add_message(&mut self, sender: impl Into<String>, text: impl Into<String>) {
        self.messages.push_back(ChatMessage {
            sender: sender.into(),
            text: text.into(),
            timestamp: Local::now(),
        });

        // Удаляем старые сообщения при превышении лимита (O(1) для VecDeque)
        if self.messages.len() > self.max_messages {
            self.messages.pop_front();
        }
    }

    /// Очищает историю
    pub fn clear(&mut self) {
        self.messages.clear();
    }

    /// Возвращает итератор по сообщениям
    pub fn messages(&self) -> impl Iterator<Item = &ChatMessage> {
        self.messages.iter()
    }
}

impl Default for ChatHistory {
    fn default() -> Self {
        Self::new(MAX_CHAT_MESSAGES)
    }
}

// ============================================================================
// Менеджер задач
// ============================================================================

/// Менеджер фоновых задач
pub struct TaskManager {
    task_sender: Sender<BackgroundTask>,
    pub result_sender: Sender<String>,
    is_processing: Arc<AtomicBool>,
}

impl TaskManager {
    /// Создаёт менеджер и возвращает канал для получения результатов
    pub fn new() -> (Self, Receiver<String>) {
        let (task_sender, task_receiver) = mpsc::channel::<BackgroundTask>();
        let (result_sender, result_receiver) = mpsc::channel::<String>();

        let result_sender_clone = result_sender.clone();
        let is_processing = Arc::new(AtomicBool::new(false));
        let is_processing_clone = is_processing.clone();

        // Фоновый поток для обработки задач
        thread::spawn(move || {
            while let Ok(task) = task_receiver.recv() {
                let result = match task {
                    BackgroundTask::SearchPackages(query) => {
                        super::commands::package::search_packages(&query)
                    }
                    BackgroundTask::InstallPackage(package) => {
                        super::commands::package::install_package(&package)
                    }
                    BackgroundTask::RemovePackage(package) => {
                        super::commands::package::remove_package(&package)
                    }
                    BackgroundTask::UpdateSystem => {
                        super::commands::package::update_system()
                    }
                    BackgroundTask::CheckYay => {
                        super::commands::package::check_yay_installed()
                    }
                    BackgroundTask::InstallYay => {
                        super::commands::package::install_yay()
                    }
                    BackgroundTask::ShutdownSystem => {
                        super::commands::system::execute_shutdown()
                    }
                    BackgroundTask::RebootSystem => {
                        super::commands::system::execute_reboot()
                    }
                    BackgroundTask::CreateCustomModel => {
                        super::ai::local_provider::create_custom_model()
                    }
                    BackgroundTask::InstallToSystem => {
                        let result = super::installer::install();
                        result.message
                    }
                    BackgroundTask::UninstallFromSystem => {
                        let result = super::installer::uninstall();
                        result.message
                    }
                };

                let _ = result_sender_clone.send(result);
                is_processing_clone.store(false, Ordering::SeqCst);
            }
        });

        (
            Self {
                task_sender,
                result_sender,
                is_processing,
            },
            result_receiver,
        )
    }

    /// Запускает фоновую задачу
    pub fn execute(&self, task: BackgroundTask) {
        // Устанавливаем флаг ДО отправки, чтобы избежать гонки
        self.is_processing.store(true, Ordering::SeqCst);
        if self.task_sender.send(task).is_err() {
            // Если отправка не удалась, сбрасываем флаг
            self.is_processing.store(false, Ordering::SeqCst);
        }
    }

    /// Проверяет, выполняется ли задача
    pub fn is_busy(&self) -> bool {
        self.is_processing.load(Ordering::SeqCst)
    }
}

// ============================================================================
// История ввода команд
// ============================================================================

const MAX_INPUT_HISTORY: usize = 50;

/// История введённых команд для навигации стрелками
pub struct InputHistory {
    entries: Vec<String>,
    position: Option<usize>,
    current_input: String,
}

impl InputHistory {
    pub fn new() -> Self {
        Self {
            entries: Vec::with_capacity(MAX_INPUT_HISTORY),
            position: None,
            current_input: String::new(),
        }
    }

    /// Добавляет команду в историю
    pub fn push(&mut self, input: &str) {
        let input = input.trim();
        if input.is_empty() {
            return;
        }
        // Не добавляем дубликаты подряд
        if self.entries.last().map(|s| s.as_str()) != Some(input) {
            self.entries.push(input.to_string());
            if self.entries.len() > MAX_INPUT_HISTORY {
                self.entries.remove(0);
            }
        }
        self.position = None;
    }

    /// Переход вверх по истории (предыдущая команда)
    pub fn up(&mut self, current: &str) -> Option<&str> {
        if self.entries.is_empty() {
            return None;
        }

        match self.position {
            None => {
                self.current_input = current.to_string();
                self.position = Some(self.entries.len() - 1);
            }
            Some(0) => return Some(&self.entries[0]),
            Some(pos) => {
                self.position = Some(pos - 1);
            }
        }

        self.position.map(|p| self.entries[p].as_str())
    }

    /// Переход вниз по истории (следующая команда)
    pub fn down(&mut self) -> Option<&str> {
        match self.position {
            None => None,
            Some(pos) => {
                if pos + 1 >= self.entries.len() {
                    self.position = None;
                    Some(self.current_input.as_str())
                } else {
                    self.position = Some(pos + 1);
                    Some(&self.entries[pos + 1])
                }
            }
        }
    }

    /// Сбрасывает позицию (при вводе нового текста)
    pub fn reset(&mut self) {
        self.position = None;
    }
}

impl Default for InputHistory {
    fn default() -> Self {
        Self::new()
    }
}
