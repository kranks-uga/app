//! Константы приложения
//!
//! Все строки, URL, настройки по умолчанию собраны здесь
//! для удобства редактирования и локализации.

// === Приложение ===
pub const APP_NAME: &str = "Альфонс";
pub const APP_VERSION: &str = "0.1.0";
pub const DEFAULT_ASSISTANT_NAME: &str = "Альфонс";
pub const DEFAULT_ACCENT_COLOR: [u8; 3] = [61, 174, 233]; // Голубой

// === Ollama AI ===
pub const OLLAMA_URL: &str = "http://localhost:11434/api/generate";
pub const OLLAMA_API_BASE: &str = "http://localhost:11434/api";
pub const OLLAMA_MODEL: &str = "llama3";
pub const OLLAMA_CUSTOM_MODEL: &str = "alfons";
pub const OLLAMA_TIMEOUT_SECS: u64 = 60;

// === Пути ===
pub const CONFIG_APP_NAME: &str = "alfons-assistant";
pub const YAY_INSTALL_DIR: &str = "/tmp/yay-install";
pub const YAY_AUR_URL: &str = "https://aur.archlinux.org/yay.git";

// === Лимиты ===
pub const MAX_CHAT_MESSAGES: usize = 100;

// === UI ===
pub const SETTINGS_PANEL_WIDTH: f32 = 280.0;

// === Сообщения ===
pub mod messages {
    pub const WELCOME: &str = "Система Arch Linux готова. Введите команду или задайте вопрос ИИ.";
    pub const CHAT_CLEARED: &str = "История чата очищена. Чем могу помочь?";
    pub const PROCESSING: &str = "Обработка...";
    pub const YAY_FOUND: &str = "yay найден и готов к работе.";
    pub const YAY_NOT_FOUND: &str = "yay не найден. Нажмите 'Установить yay' в настройках.";
    pub const YAY_INSTALLING: &str = "Начинаю установку yay... Это может занять некоторое время.";
    pub const YAY_INSTALLED: &str = "yay успешно установлен! Теперь вы можете управлять пакетами.";
    pub const YAY_ALREADY: &str = "yay уже установлен!";
    pub const MODEL_CREATING: &str = "Создаю кастомную модель 'alfons'... Это может занять несколько минут.";
    pub const MODEL_CREATED: &str = "[OK] Модель 'alfons' создана! Переключаю на неё.";
    pub const MODEL_EXISTS: &str = "Модель 'alfons' уже существует.";
}

// === Ошибки ===
pub mod errors {
    pub const OLLAMA_CONNECTION: &str = "Ошибка связи с Ollama. Убедитесь, что сервис запущен.";
    pub const OLLAMA_PARSE: &str = "Ошибка обработки ответа от Ollama.";
    pub const PACKAGE_NOT_FOUND: &str = "Ничего не найдено.";
    pub const YAY_DEPS_FAILED: &str = "Не удалось установить зависимости (git, base-devel).";
    pub const YAY_CLONE_FAILED: &str = "Не удалось клонировать репозиторий yay из AUR.";
    pub const YAY_BUILD_FAILED: &str = "Ошибка при сборке yay. Попробуйте установить вручную.";
    pub const MODEL_CREATE_FAILED: &str = "[X] Не удалось создать модель. Проверьте, что Ollama запущена и llama3 загружена.";
    pub const MODEL_BASE_NOT_FOUND: &str = "[X] Базовая модель llama3 не найдена. Выполните: ollama pull llama3";
}
