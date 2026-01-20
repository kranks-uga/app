//! Локальный AI через Ollama

use reqwest::Client;
use regex::Regex;
use serde::{Deserialize, Serialize};
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
    model: String,
    tools: ToolRegistry,
}

impl LocalAi {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(OLLAMA_TIMEOUT_SECS))
                .build()
                .unwrap_or_default(),
            model: OLLAMA_MODEL.to_string(),
            tools: ToolRegistry::new(),
        }
    }

    /// Генерирует ответ на запрос пользователя
    pub async fn generate(&self, input: &str) -> Result<String, String> {
        let payload = OllamaRequest {
            model: self.model.clone(),
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
