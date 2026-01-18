use serde::{Deserialize, Serialize};
use reqwest::Client;
use std::time::Duration;

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

pub struct LocalAi {
    client: Client,
    model: String,
}

impl LocalAi {
    pub fn new(model: &str) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(60)) // ИИ может думать долго
                .build()
                .unwrap_or_default(),
            model: model.to_string(),
        }
    }

    pub async fn generate(&self, user_input: &str) -> Result<String, String> {
        let payload = OllamaRequest {
            model: self.model.clone(),
            prompt: user_input.to_string(),
            stream: false,
            system: "Ты помощник Альфонс для Arch Linux. Отвечай кратко и по делу.".to_string(),
        };

        let res = self.client
            .post("http://localhost:11434/api/generate")
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("Ошибка связи с Ollama: {}. Убедитесь, что сервис запущен.", e))?;

        let data = res.json::<OllamaResponse>()
            .await
            .map_err(|e| format!("Ошибка обработки ответа: {}", e))?;

        Ok(data.response)
    }
}