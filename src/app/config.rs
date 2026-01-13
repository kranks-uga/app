/// Параметры персонализации ассистента
#[derive(Clone)]
pub struct Config {
    pub assistant_name: String,
    pub accent_color: [u8; 3],  // Хранение цвета в виде массива [R, G, B]
}

impl Default for Config {
    /// Установка базовых настроек (по умолчанию — стиль KDE Plasma)
    fn default() -> Self {
        Self {
            assistant_name: "Альфонс".to_string(),
            accent_color: [61, 174, 233],  // Фирменный синий цвет
        }
    }
}

impl Config {
    /// Инициализация конфигурации
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Конвертация массива байтов в тип Color32, совместимый с библиотекой egui
    pub fn accent_color_egui(&self) -> eframe::egui::Color32 {
        eframe::egui::Color32::from_rgb(
            self.accent_color[0],
            self.accent_color[1],
            self.accent_color[2],
        )
    }
}
