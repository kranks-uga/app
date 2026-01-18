use std::sync::mpsc::{self, Sender, Receiver};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

/// Типы диалоговых окон
#[derive(Debug, Clone)]
pub enum DialogType {
    PackageSearch,
    Confirmation,
    Info,
}

/// Типы фоновых задач для управления пакетами
#[derive(Debug)]
pub enum BackgroundTask {
    SearchPackages(String),
    InstallPackage(String),
    RemovePackage(String),
    UpdateSystem,
    CheckYay,
    ExecuteCommand(String),
}

/// Сообщение в чате
#[derive(Clone)]
pub struct ChatMessage {
    pub sender: String,
    pub text: String,
}

/// Управление историей чата
pub struct ChatHistory {
    messages: Vec<ChatMessage>,
    max_messages: usize,
}

impl ChatHistory {
    /// Создает новую историю чата с ограничением по количеству сообщений
    pub fn new(max_messages: usize) -> Self {
        Self {
            messages: Vec::with_capacity(max_messages),
            max_messages,
        }
    }
    
    /// Добавляет сообщение в историю
    pub fn add_message(&mut self, sender: String, text: String) {
        let message = ChatMessage { sender, text };
        self.messages.push(message);
        
        // Удаляем старые сообщения, если превышен лимит
        if self.messages.len() > self.max_messages {
            self.messages.remove(0);
        }
    }
    
    /// Очищает историю чата
    pub fn clear(&mut self) {
        self.messages.clear();
    }
    
    /// Возвращает ссылку на все сообщения
    pub fn messages(&self) -> &[ChatMessage] {
        &self.messages
    }
    
    /// Проверяет, пуста ли история
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }
    
    /// Возвращает количество сообщений
    pub fn len(&self) -> usize {
        self.messages.len()
    }
}

/// Менеджер фоновых задач
pub struct TaskManager {
    task_sender: Sender<BackgroundTask>,
    pub result_sender: Sender<String>, // Сделано публичным для доступа из assistant_app.rs
    is_processing: Arc<AtomicBool>,
}

impl TaskManager {
    /// Создает новый менеджер задач и возвращает канал для получения результатов
    pub fn new() -> (Self, Receiver<String>) {
        let (task_sender, task_receiver) = mpsc::channel::<BackgroundTask>();
        let (result_sender, result_receiver) = mpsc::channel::<String>();
        
        let result_sender_clone = result_sender.clone(); // Клон для фонового потока
        let is_processing = Arc::new(AtomicBool::new(false));
        let is_processing_clone = is_processing.clone();
        
        // Запускаем фоновый поток для обработки задач
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
                    BackgroundTask::ExecuteCommand(cmd) => {
                        super::commands::system::execute_shell_command(&cmd)
                    }
                };
                
                // Отправляем результат обратно
                let _ = result_sender_clone.send(result);
                is_processing_clone.store(false, Ordering::SeqCst);
            }
        });
        
        (
            Self {
                task_sender,
                result_sender, // Сохраняем здесь публичный отправитель
                is_processing,
            },
            result_receiver,
        )
    }
    
    /// Запускает фоновую задачу
    pub fn execute_task(&self, task: BackgroundTask) {
        if self.task_sender.send(task).is_ok() {
            self.is_processing.store(true, Ordering::SeqCst);
        }
    }
    
    /// Проверяет, выполняется ли задача
    pub fn is_processing(&self) -> bool {
        self.is_processing.load(Ordering::SeqCst)
    }
}
