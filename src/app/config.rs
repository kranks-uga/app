use serde::{Serialize, Deserialize};

/// Конфигурация с поддержкой сохранения на диск
#[derive(Serialize, Deserialize, Clone)] // Добавили Serialize и Deserialize
pub struct Config {
    pub assistant_name: String,
    pub accent_color: [u8; 3],
}

impl Default for Config {
    fn default() -> Self {
        Self {
            assistant_name: "Альфонс".to_string(),
            accent_color: [61, 174, 233],
        }
    }
}

impl Config {
    /// Загружает настройки из ~/.config/alfons-assistant/default-config.toml
    pub fn load() -> Self {
        confy::load("alfons-assistant", "config").unwrap_or_default()
    }

    /// Сохраняет текущие настройки на диск
    pub fn save(&self) {
        if let Err(e) = confy::store("alfons-assistant", "config", self) {
            eprintln!("Не удалось сохранить настройки: {}", e);
        }
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
