//! Установка приложения в систему

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Путь установки бинарника
const INSTALL_BIN_PATH: &str = ".local/bin/alfons";
/// Путь к .desktop файлу
const DESKTOP_FILE_PATH: &str = ".local/share/applications/alfons.desktop";
/// Путь к иконке
const ICON_PATH: &str = ".local/share/icons/alfons.png";

/// Результат установки
pub struct InstallResult {
    pub message: String,
}

/// Проверяет, установлено ли приложение
pub fn is_installed() -> bool {
    if let Some(home) = dirs::home_dir() {
        let bin_path = home.join(INSTALL_BIN_PATH);
        let desktop_path = home.join(DESKTOP_FILE_PATH);
        bin_path.exists() && desktop_path.exists()
    } else {
        false
    }
}

/// Получает путь к установленному бинарнику
pub fn get_installed_path() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(INSTALL_BIN_PATH))
}

/// Устанавливает приложение в систему
pub fn install() -> InstallResult {
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => {
            return InstallResult {
                message: "[X] Не удалось определить домашнюю директорию".into(),
            }
        }
    };

    // Находим текущий бинарник
    let current_exe = match std::env::current_exe() {
        Ok(p) => p,
        Err(e) => {
            return InstallResult {
                message: format!("[X] Не удалось найти исполняемый файл: {}", e),
            }
        }
    };

    // Создаём директории
    let bin_dir = home.join(".local/bin");
    let desktop_dir = home.join(".local/share/applications");
    let icon_dir = home.join(".local/share/icons");

    for dir in [&bin_dir, &desktop_dir, &icon_dir] {
        if let Err(e) = fs::create_dir_all(dir) {
            return InstallResult {
                message: format!("[X] Не удалось создать директорию: {}", e),
            };
        }
    }

    // Копируем бинарник
    let bin_path = home.join(INSTALL_BIN_PATH);
    if let Err(e) = fs::copy(&current_exe, &bin_path) {
        return InstallResult {
            message: format!("[X] Не удалось скопировать бинарник: {}", e),
        };
    }

    // Устанавливаем права на исполнение
    if let Err(e) = fs::set_permissions(&bin_path, fs::Permissions::from_mode(0o755)) {
        return InstallResult {
            message: format!("[X] Не удалось установить права: {}", e),
        };
    }

    // Ищем кастомную иконку или создаём SVG
    let icon_path = home.join(ICON_PATH);
    if let Some(custom_icon) = find_custom_icon() {
        // Копируем кастомную иконку
        if let Err(e) = fs::copy(&custom_icon, &icon_path) {
            eprintln!("Предупреждение: не удалось скопировать иконку: {}", e);
            // Fallback на SVG
            let _ = fs::write(&icon_path, generate_icon_svg());
        }
    } else {
        // Генерируем SVG иконку
        if let Err(e) = fs::write(&icon_path, generate_icon_svg()) {
            eprintln!("Предупреждение: не удалось создать иконку: {}", e);
        }
    }

    // Создаём .desktop файл
    let desktop_path = home.join(DESKTOP_FILE_PATH);
    let desktop_content = generate_desktop_file(&bin_path, &icon_path);
    if let Err(e) = fs::write(&desktop_path, desktop_content) {
        return InstallResult {
            message: format!("[X] Не удалось создать .desktop файл: {}", e),
        };
    }

    // Обновляем кэш desktop-файлов
    let _ = Command::new("update-desktop-database")
        .arg(desktop_dir)
        .output();

    InstallResult {
        message: format!(
            "[OK] Альфонс установлен!\n\
             Бинарник: {}\n\
             Ярлык добавлен в меню приложений.\n\
             Перезапустите меню или выполните: update-desktop-database",
            bin_path.display()
        ),
    }
}

/// Удаляет приложение из системы
pub fn uninstall() -> InstallResult {
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => {
            return InstallResult {
                message: "[X] Не удалось определить домашнюю директорию".into(),
            }
        }
    };

    let bin_path = home.join(INSTALL_BIN_PATH);
    let desktop_path = home.join(DESKTOP_FILE_PATH);
    let icon_path = home.join(ICON_PATH);

    let mut errors = Vec::new();

    // Удаляем файлы
    if bin_path.exists() {
        if let Err(e) = fs::remove_file(&bin_path) {
            errors.push(format!("бинарник: {}", e));
        }
    }

    if desktop_path.exists() {
        if let Err(e) = fs::remove_file(&desktop_path) {
            errors.push(format!(".desktop: {}", e));
        }
    }

    if icon_path.exists() {
        if let Err(e) = fs::remove_file(&icon_path) {
            errors.push(format!("иконка: {}", e));
        }
    }

    if errors.is_empty() {
        InstallResult {
            message: "[OK] Альфонс удалён из системы".into(),
        }
    } else {
        InstallResult {
            message: format!("[X] Ошибки при удалении: {}", errors.join(", ")),
        }
    }
}

/// Ищет кастомную иконку в стандартных местах
/// Поддерживаемые форматы: png, svg, ico
fn find_custom_icon() -> Option<PathBuf> {
    let icon_names = [
        "icon.png",
        "icon.svg",
        "alfons.png",
        "alfons.svg",
        "alfons-icon.png",
        "alfons-icon.svg",
    ];

    // Места поиска
    let mut search_paths: Vec<PathBuf> = vec![];

    // 1. Рядом с исполняемым файлом
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            search_paths.push(dir.to_path_buf());
        }
    }

    // 2. Текущая директория
    if let Ok(cwd) = std::env::current_dir() {
        search_paths.push(cwd);
    }

    // 3. Директория проекта (assets/)
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            search_paths.push(dir.join("assets"));
        }
    }
    search_paths.push(PathBuf::from("assets"));

    // 4. Конфиг директория
    if let Some(config) = dirs::config_dir() {
        search_paths.push(config.join("alfons-assistant"));
    }

    // Ищем иконку
    for path in &search_paths {
        for name in &icon_names {
            let icon_path = path.join(name);
            if icon_path.exists() {
                return Some(icon_path);
            }
        }
    }

    None
}

/// Генерирует содержимое .desktop файла
fn generate_desktop_file(bin_path: &Path, icon_path: &Path) -> String {
    format!(
        r#"[Desktop Entry]
Name=Альфонс
GenericName=AI Assistant
Comment=Помощник для Arch Linux с AI интеграцией
Exec={}
Icon={}
Terminal=false
Type=Application
Categories=Utility;System;
Keywords=arch;linux;ai;assistant;ollama;
StartupNotify=true
"#,
        bin_path.display(),
        icon_path.display()
    )
}

/// Генерирует простую SVG иконку
fn generate_icon_svg() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<svg width="128" height="128" viewBox="0 0 128 128" xmlns="http://www.w3.org/2000/svg">
  <defs>
    <linearGradient id="bg" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" style="stop-color:#3DAEE9"/>
      <stop offset="100%" style="stop-color:#1D99F3"/>
    </linearGradient>
  </defs>
  <rect width="128" height="128" rx="24" fill="url(#bg)"/>
  <text x="64" y="80" font-family="sans-serif" font-size="64" font-weight="bold"
        fill="white" text-anchor="middle">A</text>
</svg>"#
}

/// Проверяет, добавлен ли ~/.local/bin в PATH
pub fn is_local_bin_in_path() -> bool {
    if let (Some(home), Ok(path)) = (dirs::home_dir(), std::env::var("PATH")) {
        let local_bin = home.join(".local/bin");
        path.split(':').any(|p| local_bin == Path::new(p))
    } else {
        false
    }
}

/// Возвращает команду для добавления в PATH (для .bashrc/.zshrc)
pub fn get_path_export_command() -> String {
    r#"export PATH="$HOME/.local/bin:$PATH""#.to_string()
}
