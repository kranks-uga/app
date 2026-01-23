//! Локальный AI через Ollama

use super::tools::ToolRegistry;
use crate::app::constants::{
    errors, messages, OLLAMA_CUSTOM_MODEL, OLLAMA_INSTALL_SCRIPT, OLLAMA_MODEL,
    OLLAMA_TIMEOUT_SECS, OLLAMA_URL,
};
use regex::Regex;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::sync::{OnceLock, RwLock};
use std::time::Duration;

/// Статический Regex для парсинга [TOOL:...] маркеров
fn tool_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\[TOOL:([^\]]+)\]").expect("Invalid TOOL regex"))
}

#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
    system: String,
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
}

/// Клиент для работы с Ollama
pub struct LocalAi {
    client: Client,
    model: RwLock<String>,
    tools: ToolRegistry,
}

impl LocalAi {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(OLLAMA_TIMEOUT_SECS))
                .build()
                .unwrap_or_default(),
            model: RwLock::new(OLLAMA_MODEL.to_string()),
            tools: ToolRegistry::new(),
        }
    }

    /// Устанавливает модель
    pub fn set_model(&self, model: &str) {
        if let Ok(mut m) = self.model.write() {
            *m = model.to_string();
        }
    }

    /// Возвращает текущую модель
    pub fn get_model(&self) -> String {
        self.model.read().map(|m| m.clone()).unwrap_or_default()
    }

    /// Генерирует ответ на запрос пользователя
    pub async fn generate(&self, input: &str) -> Result<String, String> {
        let payload = OllamaRequest {
            model: self.get_model(),
            prompt: input.to_string(),
            stream: false,
            system: self.tools.generate_system_prompt(),
        };

        let response = self
            .client
            .post(OLLAMA_URL)
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("{}: {}", errors::OLLAMA_CONNECTION, e))?;

        let data: OllamaResponse = response
            .json()
            .await
            .map_err(|e| format!("{}: {}", errors::OLLAMA_PARSE, e))?;

        // Обрабатываем инструменты и команды в ответе
        Ok(self.process_response(&data.response))
    }

    /// Обрабатывает маркеры [TOOL:...] и [CMD:...] в ответе
    fn process_response(&self, response: &str) -> String {
        // Сначала обрабатываем TOOL маркеры
        let tool_re = tool_regex();
        let with_tools = tool_re.replace_all(response, |caps: &regex::Captures| {
            let tool = &caps[1];
            self.tools
                .execute(tool)
                .unwrap_or_else(|| format!("[?{}]", tool))
        });

        // CMD маркеры оставляем как есть - они будут обработаны в assistant_app
        with_tools.to_string()
    }
}

impl Default for LocalAi {
    fn default() -> Self {
        Self::new()
    }
}

/// Проверяет, запущен ли Ollama
pub async fn check_ollama_status() -> bool {
    let client = Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .unwrap_or_default();

    client
        .get("http://localhost:11434/api/tags")
        .send()
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false)
}

/// Проверяет, существует ли кастомная модель alfons
pub fn is_custom_model_exists() -> bool {
    Command::new("ollama")
        .args(["show", OLLAMA_CUSTOM_MODEL])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Проверяет, существует ли базовая модель llama3
pub fn is_base_model_exists() -> bool {
    Command::new("ollama")
        .args(["show", OLLAMA_MODEL])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Создаёт кастомную модель alfons из Modelfile
/// Возвращает сообщение о результате
pub fn create_custom_model() -> String {
    // Проверяем, что базовая модель существует
    if !is_base_model_exists() {
        return errors::MODEL_BASE_NOT_FOUND.to_string();
    }

    // Проверяем, не существует ли уже модель
    if is_custom_model_exists() {
        return messages::MODEL_EXISTS.to_string();
    }

    // Находим путь к Modelfile (рядом с исполняемым файлом или в текущей директории)
    let modelfile_paths = [
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.join("Modelfile")))
            .unwrap_or_default(),
        std::path::PathBuf::from("Modelfile"),
        dirs::config_dir()
            .map(|p| p.join("alfons-assistant").join("Modelfile"))
            .unwrap_or_default(),
    ];

    let modelfile = modelfile_paths.iter().find(|p| p.exists()).cloned();

    let modelfile = match modelfile {
        Some(path) => path,
        None => {
            // Создаём Modelfile в конфиг директории
            if let Some(config_dir) = dirs::config_dir() {
                let config_path = config_dir.join("alfons-assistant");
                let _ = std::fs::create_dir_all(&config_path);
                let modelfile_path = config_path.join("Modelfile");
                if std::fs::write(&modelfile_path, generate_modelfile_content()).is_ok() {
                    modelfile_path
                } else {
                    return errors::MODEL_CREATE_FAILED.to_string();
                }
            } else {
                return errors::MODEL_CREATE_FAILED.to_string();
            }
        }
    };

    // Создаём модель
    match Command::new("ollama")
        .args(["create", OLLAMA_CUSTOM_MODEL, "-f"])
        .arg(&modelfile)
        .output()
    {
        Ok(output) if output.status.success() => messages::MODEL_CREATED.to_string(),
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            format!("{} ({})", errors::MODEL_CREATE_FAILED, stderr.trim())
        }
        Err(e) => {
            format!("{} ({})", errors::MODEL_CREATE_FAILED, e)
        }
    }
}

/// Генерирует содержимое Modelfile
fn generate_modelfile_content() -> &'static str {
    r#"FROM llama3

SYSTEM """
Ты Альфонс — умный помощник для Arch Linux. Отвечай кратко и по делу на русском языке.

ДОСТУПНЫЕ ИНСТРУМЕНТЫ:
- [TOOL:время] - текущее время
- [TOOL:дата] - текущая дата
- [TOOL:память] - использование RAM
- [TOOL:диск] - использование дисков
- [TOOL:cpu] - информация о процессоре

ДОСТУПНЫЕ КОМАНДЫ (формат: [CMD:команда]):
- [CMD:очистить] - очистить чат
- [CMD:поиск <запрос>] - найти пакеты
- [CMD:установить <пакет>] - установить (откроется диалог!)
- [CMD:удалить <пакет>] - удалить (откроется диалог!)
- [CMD:обновить систему] - обновить (откроется диалог!)
- [CMD:гайд <тема>] - показать гайд (pacman, aur, wifi, systemd, gpu, audio)

ПРАВИЛА:
1. Команды установки/удаления ТОЛЬКО открывают диалог - НЕ говори "установлено"!
2. Опасные команды (выключить, перезагрузить) - ТОЛЬКО по явному запросу!
"""

PARAMETER temperature 0.7
PARAMETER top_p 0.9
PARAMETER num_ctx 4096
"#
}

/// Проверяет, установлена ли Ollama
pub fn is_ollama_installed() -> bool {
    Command::new("which")
        .arg("ollama")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Устанавливает Ollama через официальный скрипт
/// curl -fsSL https://ollama.com/install.sh | sh
pub fn install_ollama() -> String {
    // Проверяем, не установлена ли уже
    if is_ollama_installed() {
        return messages::OLLAMA_ALREADY.to_string();
    }

    // Скачиваем и выполняем установочный скрипт
    let result = Command::new("sh")
        .arg("-c")
        .arg(format!("curl -fsSL {} | sh", OLLAMA_INSTALL_SCRIPT))
        .output();

    match result {
        Ok(output) if output.status.success() => messages::OLLAMA_INSTALLED.to_string(),
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            format!("{} ({})", errors::OLLAMA_INSTALL_FAILED, stderr.trim())
        }
        Err(e) => {
            format!("{} ({})", errors::OLLAMA_INSTALL_FAILED, e)
        }
    }
}

/// Запускает сервис Ollama в фоне
pub fn start_ollama_service() -> String {
    // Проверяем, установлена ли Ollama
    if !is_ollama_installed() {
        return errors::OLLAMA_INSTALL_FAILED.to_string();
    }

    // Запускаем ollama serve в фоне
    let result = Command::new("sh").arg("-c").arg("ollama serve &").spawn();

    match result {
        Ok(_) => {
            // Даём серверу время запуститься
            std::thread::sleep(std::time::Duration::from_secs(2));
            messages::OLLAMA_STARTED.to_string()
        }
        Err(e) => {
            format!("{} ({})", errors::OLLAMA_START_FAILED, e)
        }
    }
}
