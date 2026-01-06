/// Конфигурация приложения
#[derive(Clone)]
pub struct Config {
    pub assistant_name: String,
    pub accent_color: [u8; 3],  // RGB цвет
}

impl Default for Config {
    fn default() -> Self {
        Self {
            assistant_name: "Альфонс".to_string(),
            accent_color: [61, 174, 233],  // Синий KDE
        }
    }
}

impl Config {
    /// Создает новую конфигурацию с настройками по умолчанию
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Возвращает цвет акцента в формате egui::Color32
    pub fn accent_color_egui(&self) -> eframe::egui::Color32 {
        eframe::egui::Color32::from_rgb(
            self.accent_color[0],
            self.accent_color[1],
            self.accent_color[2],
        )
    }
}