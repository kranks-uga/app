//! Константы приложения
//!
//! Все строки, URL, настройки по умолчанию собраны здесь
//! для удобства редактирования и локализации.

// === Приложение ===
pub const APP_NAME: &str = "Альфонс";
pub const APP_VERSION: &str = "0.0.5";
pub const DEFAULT_ASSISTANT_NAME: &str = "Альфонс";
pub const DEFAULT_ACCENT_COLOR: [u8; 3] = [61, 174, 233]; // Голубой

// === Ollama AI ===
pub const OLLAMA_URL: &str = "http://localhost:11434/api/generate";
pub const OLLAMA_API_BASE: &str = "http://localhost:11434/api";
pub const OLLAMA_MODEL: &str = "llama3";
pub const OLLAMA_CUSTOM_MODEL: &str = "alfons";
pub const OLLAMA_TIMEOUT_SECS: u64 = 60;
pub const OLLAMA_INSTALL_SCRIPT: &str = "https://ollama.com/install.sh";

// === Yay (AUR) ===
pub const YAY_INSTALL_DIR: &str = "/tmp/yay-install";
pub const YAY_AUR_URL: &str = "https://aur.archlinux.org/yay.git";

// === Пути ===
pub const CONFIG_APP_NAME: &str = "alfons-assistant";

// === Лимиты ===
pub const MAX_CHAT_MESSAGES: usize = 100;

// === UI ===
pub const SETTINGS_PANEL_WIDTH: f32 = 280.0;

// === Сообщения ===
pub mod messages {
    pub const WELCOME: &str = "Система готова. Введите команду или задайте вопрос ИИ.";
    pub const CHAT_CLEARED: &str = "История чата очищена. Чем могу помочь?";
    pub const PROCESSING: &str = "Обработка...";
    pub const FLATPAK_FOUND: &str = "flatpak найден и готов к работе.";
    pub const FLATPAK_NOT_FOUND: &str = "flatpak не найден. Установите его через пакетный менеджер.";
    pub const MODEL_CREATING: &str = "Создаю кастомную модель 'alfons'... Это может занять несколько минут.";
    pub const MODEL_CREATED: &str = "[OK] Модель 'alfons' создана! Переключаю на неё.";
    pub const MODEL_EXISTS: &str = "Модель 'alfons' уже существует.";
    pub const OLLAMA_INSTALLING: &str = "Устанавливаю Ollama... Это может занять некоторое время.";
    pub const OLLAMA_INSTALLED: &str = "[OK] Ollama успешно установлена!";
    pub const OLLAMA_ALREADY: &str = "Ollama уже установлена!";
    pub const OLLAMA_STARTING: &str = "Запускаю сервис Ollama...";
    pub const OLLAMA_STARTED: &str = "[OK] Сервис Ollama запущен!";
    pub const YAY_INSTALLING: &str = "Устанавливаю yay... Это может занять некоторое время.";
    pub const YAY_INSTALLED: &str = "[OK] yay успешно установлен!";
    pub const YAY_FOUND: &str = "[OK] yay найден и готов к работе.";
    pub const YAY_NOT_FOUND: &str = "yay не найден. Нажмите кнопку для установки.";
    pub const YAY_ALREADY: &str = "yay уже установлен!";
}

// === Ошибки ===
pub mod errors {
    pub const OLLAMA_CONNECTION: &str = "Ошибка связи с Ollama. Убедитесь, что сервис запущен.";
    pub const OLLAMA_PARSE: &str = "Ошибка обработки ответа от Ollama.";
    pub const PACKAGE_NOT_FOUND: &str = "Ничего не найдено.";
    pub const MODEL_CREATE_FAILED: &str = "[X] Не удалось создать модель. Проверьте, что Ollama запущена и llama3 загружена.";
    pub const MODEL_BASE_NOT_FOUND: &str = "[X] Базовая модель llama3 не найдена. Выполните: ollama pull llama3";
    pub const OLLAMA_INSTALL_FAILED: &str = "[X] Не удалось установить Ollama.";
    pub const OLLAMA_START_FAILED: &str = "[X] Не удалось запустить сервис Ollama.";
    pub const YAY_INSTALL_FAILED: &str = "[X] Не удалось установить yay.";
    pub const YAY_DEPS_FAILED: &str = "[X] Не удалось установить зависимости для yay.";
    pub const YAY_CLONE_FAILED: &str = "[X] Не удалось склонировать репозиторий yay.";
    pub const YAY_BUILD_FAILED: &str = "[X] Не удалось собрать yay.";
}
