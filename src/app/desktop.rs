//! Определение окружения рабочего стола и адаптация UI

use std::env;
use std::process::Command;

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
            if desktop.contains("gnome") || desktop.contains("unity") || desktop.contains("budgie")
            {
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
                "kgx", // GNOME Console
                "alacritty",
                "kitty",
                "xterm",
            ],
            Self::Kde => vec!["konsole", "alacritty", "kitty", "xterm"],
            Self::Xfce => vec!["xfce4-terminal", "alacritty", "kitty", "xterm"],
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
}

impl DeStyles {
    /// Получить стили для текущего DE
    pub fn for_de(de: DesktopEnvironment) -> Self {
        match de {
            DesktopEnvironment::Gnome => Self {
                rounding: 12.0, // GNOME использует более округлые формы
                spacing: 12.0,
            },
            DesktopEnvironment::Kde => Self {
                rounding: 6.0, // KDE более строгий
                spacing: 10.0,
            },
            DesktopEnvironment::Xfce => Self {
                rounding: 4.0, // Xfce минималистичный
                spacing: 8.0,
            },
            DesktopEnvironment::Other => Self {
                rounding: 8.0,
                spacing: 10.0,
            },
        }
    }
}

pub fn run_in_terminal(cmd: &str, action: &str) -> String {
    let de = DesktopEnvironment::detect();
    let terminals = de.terminal_priority();

    for term in terminals {
        // Проверяем, установлен ли терминал
        if !Command::new("which")
            .arg(term)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            continue;
        }

        // Получаем аргументы для терминала
        let args = match get_terminal_args(term, cmd) {
            Some(a) => a,
            None => continue,
        };

        // Запускаем
        match Command::new(term).args(&args).spawn() {
            Ok(_) => return format!("[OK] {} запущено в {}", action, term),
            Err(_) => continue,
        }
    }

    format!(
        "[X] Не найден терминал для {}. Установите {} или другой терминал.",
        de.name(),
        de.preferred_terminal()
    )
}

/// Возвращает аргументы для запуска команды в конкретном терминале
fn get_terminal_args(term: &str, cmd: &str) -> Option<Vec<String>> {
    let args = match term {
        "kitty" => vec![
            "--hold".to_string(),
            "-e".to_string(),
            "sh".to_string(),
            "-c".to_string(),
            cmd.to_string(),
        ],
        "alacritty" => vec![
            "-e".to_string(),
            "sh".to_string(),
            "-c".to_string(),
            format!("{}; echo 'Нажмите Enter...'; read", cmd),
        ],
        "gnome-terminal" | "kgx" => vec![
            "--".to_string(),
            "sh".to_string(),
            "-c".to_string(),
            format!("{}; echo 'Нажмите Enter...'; read", cmd),
        ],
        "konsole" => vec![
            "-e".to_string(),
            "sh".to_string(),
            "-c".to_string(),
            format!("{}; echo 'Нажмите Enter...'; read", cmd),
        ],
        "xfce4-terminal" => vec![
            "-e".to_string(),
            format!("sh -c '{}; echo Нажмите Enter...; read'", cmd),
        ],
        "xterm" => vec![
            "-hold".to_string(),
            "-e".to_string(),
            "sh".to_string(),
            "-c".to_string(),
            cmd.to_string(),
        ],
        _ => return None,
    };
    Some(args)
}
