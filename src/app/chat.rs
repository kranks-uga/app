//! Модуль чата и фоновых задач

use super::constants::{CHATS_DIR, CONFIG_APP_NAME, MAX_CHAT_MESSAGES, MAX_SESSION_TITLE_LEN};
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::thread;
use uuid::Uuid;

// ============================================================================
// Диалоги
// ============================================================================

/// Информация об одном найденном пакете
#[derive(Debug, Clone)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
    pub repo: String,
    pub description: String,
}

/// Типы диалоговых окон
#[derive(Debug, Clone, Default, PartialEq)]
pub enum DialogType {
    #[default]
    Info,
    PackageSearch,
    PackageResults,
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
    pub search_results: Vec<PackageInfo>,
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

    /// Показать результаты поиска пакетов
    pub fn show_package_results(&mut self, query: &str, results: Vec<PackageInfo>) {
        self.visible = true;
        self.dialog_type = DialogType::PackageResults;
        self.title = format!("Найдено {} пакетов: \"{}\"", results.len(), query);
        self.message = String::new();
        self.search_results = results;
    }

    /// Скрыть диалог
    pub fn hide(&mut self) {
        self.visible = false;
        self.input.clear();
        self.package.clear();
        self.search_results.clear();
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
    InstallYay,
    ShutdownSystem,
    RebootSystem,
    CreateCustomModel,
    InstallToSystem,
    UninstallFromSystem,
    InstallOllama,
    StartOllama,
}

// ============================================================================
// История чата
// ============================================================================

/// Сообщение в чате
#[derive(Clone, Serialize, Deserialize)]
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

    /// Возвращает историю как вектор пар (отправитель, текст)
    pub fn as_pairs(&self) -> Vec<(String, String)> {
        self.messages
            .iter()
            .map(|m| (m.sender.clone(), m.text.clone()))
            .collect()
    }

    /// Возвращает все сообщения как Vec для сериализации
    pub fn to_vec(&self) -> Vec<ChatMessage> {
        self.messages.iter().cloned().collect()
    }

    /// Загружает сообщения из вектора
    pub fn load_from(&mut self, messages: Vec<ChatMessage>) {
        self.messages.clear();
        for msg in messages {
            self.messages.push_back(msg);
        }
        // Обрезаем если превышен лимит
        while self.messages.len() > self.max_messages {
            self.messages.pop_front();
        }
    }

}

impl Default for ChatHistory {
    fn default() -> Self {
        Self::new(MAX_CHAT_MESSAGES)
    }
}

// ============================================================================
// Сессии чатов
// ============================================================================

/// Полная сессия чата (для сохранения на диск)
#[derive(Serialize, Deserialize, Clone)]
pub struct ChatSession {
    pub id: String,
    pub title: String,
    pub created_at: DateTime<Local>,
    pub messages: Vec<ChatMessage>,
}

/// Метаданные сессии (для списка в sidebar, без сообщений)
#[derive(Serialize, Deserialize, Clone)]
pub struct ChatSessionMeta {
    pub id: String,
    pub title: String,
    pub created_at: DateTime<Local>,
}

/// Менеджер сессий чатов
pub struct ChatSessionManager {
    pub sessions: Vec<ChatSessionMeta>,
    pub current_session_id: Option<String>,
    chats_dir: PathBuf,
}

impl ChatSessionManager {
    /// Создаёт менеджер, инициализирует директорию, загружает список метаданных
    pub fn new() -> Self {
        let chats_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(CONFIG_APP_NAME)
            .join(CHATS_DIR);

        // Создаём директорию если не существует
        let _ = fs::create_dir_all(&chats_dir);

        let mut manager = Self {
            sessions: Vec::new(),
            current_session_id: None,
            chats_dir,
        };
        manager.load_meta_list();
        manager
    }

    /// Создаёт новую сессию, возвращает её id
    pub fn create_session(&mut self) -> String {
        let id = Uuid::new_v4().to_string();
        let meta = ChatSessionMeta {
            id: id.clone(),
            title: "Новый чат".to_string(),
            created_at: Local::now(),
        };

        // Сохраняем пустую сессию на диск
        let session = ChatSession {
            id: id.clone(),
            title: meta.title.clone(),
            created_at: meta.created_at,
            messages: Vec::new(),
        };
        self.write_session(&session);

        self.sessions.insert(0, meta);
        self.current_session_id = Some(id.clone());
        id
    }

    /// Сохраняет текущий чат на диск
    pub fn save_session(&self, id: &str, chat: &ChatHistory) {
        if let Some(meta) = self.sessions.iter().find(|s| s.id == id) {
            let session = ChatSession {
                id: id.to_string(),
                title: meta.title.clone(),
                created_at: meta.created_at,
                messages: chat.to_vec(),
            };
            self.write_session(&session);
        }
    }

    /// Загружает чат с диска в ChatHistory
    pub fn load_session(&self, id: &str) -> Option<ChatHistory> {
        let path = self.chats_dir.join(format!("{}.json", id));
        let data = fs::read_to_string(&path).ok()?;
        let session: ChatSession = serde_json::from_str(&data).ok()?;

        let mut history = ChatHistory::default();
        history.load_from(session.messages);
        Some(history)
    }

    /// Удаляет сессию (файл и метаданные)
    pub fn delete_session(&mut self, id: &str) {
        let path = self.chats_dir.join(format!("{}.json", id));
        let _ = fs::remove_file(&path);
        self.sessions.retain(|s| s.id != id);
    }

    /// Обновляет заголовок сессии
    pub fn update_title(&mut self, id: &str, title: &str) {
        let truncated: String = title.chars().take(MAX_SESSION_TITLE_LEN).collect();
        if let Some(meta) = self.sessions.iter_mut().find(|s| s.id == id) {
            meta.title = truncated;
        }
    }

    /// Сканирует директорию и загружает метаданные всех сессий
    fn load_meta_list(&mut self) {
        self.sessions.clear();

        let entries = match fs::read_dir(&self.chats_dir) {
            Ok(e) => e,
            Err(_) => return,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                if let Ok(data) = fs::read_to_string(&path) {
                    if let Ok(session) = serde_json::from_str::<ChatSession>(&data) {
                        self.sessions.push(ChatSessionMeta {
                            id: session.id,
                            title: session.title,
                            created_at: session.created_at,
                        });
                    }
                }
            }
        }

        // Сортируем по дате создания (новые сверху)
        self.sessions
            .sort_by(|a, b| b.created_at.cmp(&a.created_at));
    }

    /// Записывает сессию на диск
    fn write_session(&self, session: &ChatSession) {
        let path = self.chats_dir.join(format!("{}.json", session.id));
        if let Ok(json) = serde_json::to_string_pretty(session) {
            let _ = fs::write(&path, json);
        }
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
                    BackgroundTask::UpdateSystem => super::commands::package::update_system(),
                    BackgroundTask::InstallYay => super::commands::package::install_yay(),
                    BackgroundTask::ShutdownSystem => super::commands::system::execute_shutdown(),
                    BackgroundTask::RebootSystem => super::commands::system::execute_reboot(),
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
                    BackgroundTask::InstallOllama => super::ai::local_provider::install_ollama(),
                    BackgroundTask::StartOllama => {
                        super::ai::local_provider::start_ollama_service()
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
