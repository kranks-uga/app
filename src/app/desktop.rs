//! Определение окружения рабочего стола и адаптация UI

use std::env;

/// Тип окружения рабочего стола
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum DesktopEnvironment {
    Gnome,
    Kde,
    Xfce,
    #[default]
    Other,
}

impl DesktopEnvironment {
    /// Определяет текущее окружение рабочего стола
    pub fn detect() -> Self {
        // Проверяем XDG_CURRENT_DESKTOP
        if let Ok(desktop) = env::var("XDG_CURRENT_DESKTOP") {
            let desktop = desktop.to_lowercase();
            if desktop.contains("gnome") || desktop.contains("unity") || desktop.contains("budgie") {
                return Self::Gnome;
            }
            if desktop.contains("kde") || desktop.contains("plasma") {
                return Self::Kde;
            }
            if desktop.contains("xfce") {
                return Self::Xfce;
            }
        }

        // Проверяем DESKTOP_SESSION
        if let Ok(session) = env::var("DESKTOP_SESSION") {
            let session = session.to_lowercase();
            if session.contains("gnome") || session.contains("ubuntu") {
                return Self::Gnome;
            }
            if session.contains("plasma") || session.contains("kde") {
                return Self::Kde;
            }
            if session.contains("xfce") {
                return Self::Xfce;
            }
        }

        // Проверяем KDE_FULL_SESSION
        if env::var("KDE_FULL_SESSION").is_ok() {
            return Self::Kde;
        }

        // Проверяем GNOME_DESKTOP_SESSION_ID
        if env::var("GNOME_DESKTOP_SESSION_ID").is_ok() {
            return Self::Gnome;
        }

        Self::Other
    }

    /// Возвращает предпочтительный терминал для данного DE
    pub fn preferred_terminal(&self) -> &'static str {
        match self {
            Self::Gnome => "gnome-terminal",
            Self::Kde => "konsole",
            Self::Xfce => "xfce4-terminal",
            Self::Other => "xterm",
        }
    }

    /// Возвращает список терминалов в порядке приоритета для данного DE
    pub fn terminal_priority(&self) -> Vec<&'static str> {
        match self {
            Self::Gnome => vec![
                "gnome-terminal",
                "kgx",           // GNOME Console
                "alacritty",
                "kitty",
                "xterm",
            ],
            Self::Kde => vec![
                "konsole",
                "alacritty",
                "kitty",
                "xterm",
            ],
            Self::Xfce => vec![
                "xfce4-terminal",
                "alacritty",
                "kitty",
                "xterm",
            ],
            Self::Other => vec![
                "alacritty",
                "kitty",
                "gnome-terminal",
                "konsole",
                "xfce4-terminal",
                "xterm",
            ],
        }
    }

    /// Название DE для отображения
    pub fn name(&self) -> &'static str {
        match self {
            Self::Gnome => "GNOME",
            Self::Kde => "KDE Plasma",
            Self::Xfce => "Xfce",
            Self::Other => "Linux",
        }
    }
}

/// Стили UI для разных DE
pub struct DeStyles {
    pub rounding: f32,
    pub spacing: f32,
    pub button_padding: f32,
}

impl DeStyles {
    /// Получить стили для текущего DE
    pub fn for_de(de: DesktopEnvironment) -> Self {
        match de {
            DesktopEnvironment::Gnome => Self {
                rounding: 12.0,      // GNOME использует более округлые формы
                spacing: 12.0,
                button_padding: 12.0,
            },
            DesktopEnvironment::Kde => Self {
                rounding: 6.0,       // KDE более строгий
                spacing: 10.0,
                button_padding: 10.0,
            },
            DesktopEnvironment::Xfce => Self {
                rounding: 4.0,       // Xfce минималистичный
                spacing: 8.0,
                button_padding: 8.0,
            },
            DesktopEnvironment::Other => Self {
                rounding: 8.0,
                spacing: 10.0,
                button_padding: 10.0,
            },
        }
    }
}

/// Цвета по умолчанию для разных DE
pub fn default_accent_color(de: DesktopEnvironment) -> [u8; 3] {
    match de {
        DesktopEnvironment::Gnome => [53, 132, 228],   // GNOME синий
        DesktopEnvironment::Kde => [61, 174, 233],     // KDE голубой
        DesktopEnvironment::Xfce => [44, 137, 218],    // Xfce синий
        DesktopEnvironment::Other => [61, 174, 233],   // По умолчанию
    }
}
