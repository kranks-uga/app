//! Команды управления пакетами (через yay)

use crate::app::chat::{BackgroundTask, DialogState, TaskManager};
use crate::app::constants::{errors, messages, YAY_AUR_URL, YAY_INSTALL_DIR};
use crate::app::desktop::run_in_terminal;
use std::process::Command;

/// Маппинг русских названий приложений на реальные имена пакетов
fn resolve_package_alias(input: &str) -> &str {
    match input {
        "стим" | "steam" => "steam",
        "дискорд" | "discord" => "discord",
        "телеграм" | "телеграмм" | "telegram" => "telegram-desktop",
        "вс код" | "vs code" | "vscode" => "visual-studio-code-bin",
        "фаерфокс" | "firefox" => "firefox",
        "хром" | "хромиум" | "chromium" => "chromium",
        "влц" | "vlc" => "vlc",
        "гимп" | "gimp" => "gimp",
        "обс" | "obs" => "obs-studio",
        "лутрис" | "lutris" => "lutris",
        other => other,
    }
}

/// Проверяет, что имя пакета содержит только допустимые символы pacman/AUR
fn is_valid_package_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '@' | '.' | '_' | '+' | '-'))
}

/// Обработка команд пакетного менеджера
pub fn process_package_command(
    cmd: &str,
    dialog: &mut DialogState,
    tasks: &TaskManager,
) -> Option<String> {
    // Открыть диалог поиска
    if cmd == "поиск пакетов" || cmd == "найти пакеты" {
        dialog.show_search();
        return Some("Открываю поиск пакетов...".into());
    }

    // Установка: "установить <пакет>"
    if let Some(package) = cmd.strip_prefix("установить ") {
        let package = package.trim();
        if package.is_empty() {
            return Some("Укажите пакет. Пример: установить firefox".into());
        }
        let package = resolve_package_alias(package);
        dialog.show_confirm(
            "Установка пакета",
            &format!("Установить '{}' через yay?", package),
            package,
        );
        return Some(format!("Подготовка к установке '{}'...", package));
    }

    // Удаление: "удалить <пакет>"
    if let Some(package) = cmd.strip_prefix("удалить ") {
        let package = package.trim();
        if package.is_empty() {
            return Some("Укажите пакет для удаления.".into());
        }
        let package = resolve_package_alias(package);
        dialog.show_confirm(
            "Удаление пакета",
            &format!("Удалить '{}' из системы?", package),
            package,
        );
        return Some(format!("Подготовка к удалению '{}'...", package));
    }

    // Обновление системы
    if cmd == "обновить систему"
        || cmd == "обновить система"
        || cmd == "обновление"
        || cmd == "обновить"
    {
        dialog.show_confirm(
            "Обновление системы",
            "Выполнить полное обновление (yay)?",
            "",
        );
        return Some("Подготовка к обновлению...".into());
    }

    // Быстрый поиск: "поиск <запрос>"
    if let Some(query) = cmd.strip_prefix("поиск ") {
        let query = query.trim();
        if !query.is_empty() {
            tasks.execute(BackgroundTask::SearchPackages(query.into()));
            return Some(format!("Ищу пакеты '{}'...", query));
        }
    }

    None
}

// ============================================================================
// Функции выполнения (вызываются из фонового потока)
// ============================================================================

/// Поиск пакетов через yay
pub fn search_packages(query: &str) -> String {
    if !is_valid_package_name(query) {
        return "Недопустимые символы в запросе поиска.".into();
    }
    match Command::new("yay").args(["-Ss", query]).output() {
        Ok(out) => {
            let result = String::from_utf8_lossy(&out.stdout);
            if result.trim().is_empty() {
                errors::PACKAGE_NOT_FOUND.into()
            } else {
                result.into()
            }
        }
        Err(e) => format!("Ошибка yay: {}", e),
    }
}

/// Проверяет, требует ли пакет включённого репозитория multilib
fn needs_multilib(package: &str) -> bool {
    matches!(package, "steam" | "lib32-mesa" | "lib32-vulkan-icd-loader")
}

/// Проверяет, включён ли репозиторий [multilib] в pacman.conf
fn is_multilib_enabled() -> bool {
    std::fs::read_to_string("/etc/pacman.conf")
        .map(|content| content.lines().any(|line| line.trim() == "[multilib]"))
        .unwrap_or(false)
}

/// Включает репозиторий [multilib] через pkexec и обновляет базы пакетов
fn enable_multilib() -> Result<(), String> {
    let sed = Command::new("pkexec")
        .args([
            "sed",
            "-i",
            "/^#\\[multilib\\]/,/^#Include/ s/^#//",
            "/etc/pacman.conf",
        ])
        .status();
    if sed.is_err() || !sed.unwrap().success() {
        return Err("Не удалось включить multilib.".into());
    }
    let sync = Command::new("pkexec")
        .args(["pacman", "-Sy"])
        .status();
    if sync.is_err() || !sync.unwrap().success() {
        return Err("Не удалось обновить базы пакетов.".into());
    }
    Ok(())
}

/// Установка пакета
/// Запускаем в терминале для интерактивного sudo
pub fn install_package(package: &str) -> String {
    if !is_valid_package_name(package) {
        return "Недопустимое имя пакета.".into();
    }
    if needs_multilib(package) && !is_multilib_enabled() {
        if let Err(e) = enable_multilib() {
            return e;
        }
    }
    run_in_terminal(
        &format!("yay -S {}", package),
        &format!("Установка {}", package),
    )
}

/// Удаление пакета
/// Запускаем в терминале для интерактивного sudo
pub fn remove_package(package: &str) -> String {
    if !is_valid_package_name(package) {
        return "Недопустимое имя пакета.".into();
    }
    run_in_terminal(
        &format!("yay -R {}", package),
        &format!("Удаление {}", package),
    )
}

/// Обновление системы
/// Запускаем в терминале, т.к. yay требует интерактивный ввод для sudo
pub fn update_system() -> String {
    run_in_terminal("yay", "Обновление")
}

/// Проверка yay (возвращает bool)
pub fn is_yay_installed() -> bool {
    Command::new("which")
        .arg("yay")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Установка yay из AUR
pub fn install_yay() -> String {
    if is_yay_installed() {
        return messages::YAY_ALREADY.into();
    }

    // 1. Установка зависимостей
    let deps = Command::new("pkexec")
        .args([
            "pacman",
            "-S",
            "--needed",
            "--noconfirm",
            "git",
            "base-devel",
        ])
        .status();

    if deps.is_err() || !deps.unwrap().success() {
        return errors::YAY_DEPS_FAILED.into();
    }

    // 2. Клонирование репозитория
    let _ = std::fs::remove_dir_all(YAY_INSTALL_DIR);

    let clone = Command::new("git")
        .args(["clone", YAY_AUR_URL, YAY_INSTALL_DIR])
        .status();

    if clone.is_err() || !clone.unwrap().success() {
        return errors::YAY_CLONE_FAILED.into();
    }

    // 3. Сборка и установка
    let build = Command::new("sh")
        .args([
            "-c",
            &format!("cd {} && makepkg -si --noconfirm", YAY_INSTALL_DIR),
        ])
        .status();

    // Очистка
    let _ = std::fs::remove_dir_all(YAY_INSTALL_DIR);

    match build {
        Ok(s) if s.success() && is_yay_installed() => messages::YAY_INSTALLED.into(),
        _ => errors::YAY_BUILD_FAILED.into(),
    }
}
