//! Локальный AI через Ollama

use reqwest::Client;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::RwLock;
use std::time::Duration;
use super::tools::ToolRegistry;
use crate::app::constants::{OLLAMA_URL, OLLAMA_MODEL, OLLAMA_TIMEOUT_SECS, errors};

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
        let tool_re = Regex::new(r"\[TOOL:([^\]]+)\]").unwrap();
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

/// Получает список доступных моделей Ollama
pub async fn get_available_models() -> Vec<String> {
    #[derive(Deserialize)]
    struct ModelsResponse {
        models: Vec<ModelInfo>,
    }
    #[derive(Deserialize)]
    struct ModelInfo {
        name: String,
    }

    let client = Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap_or_default();

    match client.get("http://localhost:11434/api/tags").send().await {
        Ok(resp) => {
            if let Ok(data) = resp.json::<ModelsResponse>().await {
                data.models.into_iter().map(|m| m.name).collect()
            } else {
                vec![]
            }
        }
        Err(_) => vec![],
    }
}
