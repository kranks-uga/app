//! Команды управления пакетами (через yay)

use std::process::Command;
use crate::app::chat::{BackgroundTask, DialogState, TaskManager};
use crate::app::constants::{YAY_INSTALL_DIR, YAY_AUR_URL, messages, errors};

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
        dialog.show_confirm(
            "Удаление пакета",
            &format!("Удалить '{}' из системы?", package),
            package,
        );
        return Some(format!("Подготовка к удалению '{}'...", package));
    }

    // Обновление системы
    if cmd == "обновить систему" || cmd == "обновить система" || cmd == "обновление" || cmd == "обновить" {
        dialog.show_confirm(
            "Обновление системы",
            "Выполнить полное обновление (yay -Syu)?",
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

/// Установка пакета
/// yay сам запросит sudo при необходимости, pkexec не нужен
pub fn install_package(package: &str) -> String {
    match Command::new("yay")
        .args(["-S", "--noconfirm", package])
        .status()
    {
        Ok(s) if s.success() => format!("[OK] Пакет '{}' установлен.", package),
        Ok(s) => format!("[X] Не удалось установить '{}' (код: {:?}).", package, s.code()),
        Err(e) => format!("[X] Ошибка запуска yay: {}", e),
    }
}

/// Удаление пакета
/// yay сам запросит sudo при необходимости
pub fn remove_package(package: &str) -> String {
    match Command::new("yay")
        .args(["-R", "--noconfirm", package])
        .status()
    {
        Ok(s) if s.success() => format!("[OK] Пакет '{}' удалён.", package),
        Ok(s) => format!("[X] Не удалось удалить '{}' (код: {:?}).", package, s.code()),
        Err(e) => format!("[X] Ошибка запуска yay: {}", e),
    }
}

/// Обновление системы
/// yay сам запросит sudo при необходимости
pub fn update_system() -> String {
    match Command::new("yay")
        .args(["-Syu", "--noconfirm"])
        .status()
    {
        Ok(s) if s.success() => "[OK] Система обновлена!".into(),
        Ok(s) => format!("[X] Ошибка при обновлении (код: {:?}).", s.code()),
        Err(e) => format!("[X] Ошибка запуска yay: {}", e),
    }
}

/// Проверка наличия yay
pub fn check_yay_installed() -> String {
    if is_yay_installed() {
        messages::YAY_FOUND.into()
    } else {
        messages::YAY_NOT_FOUND.into()
    }
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
        .args(["pacman", "-S", "--needed", "--noconfirm", "git", "base-devel"])
        .status();

    if deps.is_err() || !deps.unwrap().success() {
        return errors::YAY_DEPS_FAILED.into();
    }

    // 2. Клонирование репозитория
    let _ = Command::new("rm").args(["-rf", YAY_INSTALL_DIR]).status();

    let clone = Command::new("git")
        .args(["clone", YAY_AUR_URL, YAY_INSTALL_DIR])
        .status();

    if clone.is_err() || !clone.unwrap().success() {
        return errors::YAY_CLONE_FAILED.into();
    }

    // 3. Сборка и установка
    let build = Command::new("sh")
        .args(["-c", &format!("cd {} && makepkg -si --noconfirm", YAY_INSTALL_DIR)])
        .status();

    // Очистка
    let _ = Command::new("rm").args(["-rf", YAY_INSTALL_DIR]).status();

    match build {
        Ok(s) if s.success() && is_yay_installed() => messages::YAY_INSTALLED.into(),
        _ => errors::YAY_BUILD_FAILED.into(),
    }
}
