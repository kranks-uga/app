//! Конфигурация пользователя

use serde::{Deserialize, Serialize};
use super::constants::{CONFIG_APP_NAME, DEFAULT_ASSISTANT_NAME, DEFAULT_ACCENT_COLOR, OLLAMA_MODEL};

/// Настройки приложения (сохраняются на диск)
#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub assistant_name: String,
    pub accent_color: [u8; 3],
    #[serde(default = "default_ollama_model")]
    pub ollama_model: String,
}

fn default_ollama_model() -> String {
    OLLAMA_MODEL.to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            assistant_name: DEFAULT_ASSISTANT_NAME.to_string(),
            accent_color: DEFAULT_ACCENT_COLOR,
            ollama_model: OLLAMA_MODEL.to_string(),
        }
    }
}

impl Config {
    /// Загружает из ~/.config/alfons-assistant/config.toml
    pub fn load() -> Self {
        confy::load(CONFIG_APP_NAME, "config").unwrap_or_default()
    }

    /// Сохраняет на диск. Возвращает Ok(()) при успехе или сообщение об ошибке.
    pub fn save(&self) -> Result<(), String> {
        confy::store(CONFIG_APP_NAME, "config", self)
            .map_err(|e| format!("Не удалось сохранить настройки: {}", e))
    }

    /// Конвертация для egui
    pub fn accent_color_egui(&self) -> eframe::egui::Color32 {
        eframe::egui::Color32::from_rgb(
            self.accent_color[0],
            self.accent_color[1],
            self.accent_color[2],
        )
    }
}
