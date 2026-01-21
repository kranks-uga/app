//! Команды управления пакетами (через yay)

use std::process::Command;
use crate::app::chat::{BackgroundTask, DialogState, TaskManager};
use crate::app::constants::{YAY_INSTALL_DIR, YAY_AUR_URL, messages, errors};
use crate::app::desktop::DesktopEnvironment;

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
/// Запускаем в терминале для интерактивного sudo
pub fn install_package(package: &str) -> String {
    run_in_terminal(&format!("yay -S {}", package), &format!("Установка {}", package))
}

/// Удаление пакета
/// Запускаем в терминале для интерактивного sudo
pub fn remove_package(package: &str) -> String {
    run_in_terminal(&format!("yay -R {}", package), &format!("Удаление {}", package))
}

/// Возвращает аргументы для запуска команды в конкретном терминале
fn get_terminal_args(term: &str, cmd: &str) -> Option<Vec<String>> {
    let args = match term {
        "kitty" => vec!["--hold".to_string(), "-e".to_string(), "sh".to_string(), "-c".to_string(), cmd.to_string()],
        "alacritty" => vec!["-e".to_string(), "sh".to_string(), "-c".to_string(), format!("{}; echo 'Нажмите Enter...'; read", cmd)],
        "gnome-terminal" | "kgx" => vec!["--".to_string(), "sh".to_string(), "-c".to_string(), format!("{}; echo 'Нажмите Enter...'; read", cmd)],
        "konsole" => vec!["-e".to_string(), "sh".to_string(), "-c".to_string(), format!("{}; echo 'Нажмите Enter...'; read", cmd)],
        "xfce4-terminal" => vec!["-e".to_string(), format!("sh -c '{}; echo Нажмите Enter...; read'", cmd)],
        "xterm" => vec!["-hold".to_string(), "-e".to_string(), "sh".to_string(), "-c".to_string(), cmd.to_string()],
        _ => return None,
    };
    Some(args)
}

/// Запускает команду в терминале (с учётом текущего DE)
fn run_in_terminal(cmd: &str, action: &str) -> String {
    let de = DesktopEnvironment::detect();
    let terminals = de.terminal_priority();

    for term in terminals {
        // Проверяем, установлен ли терминал
        if !Command::new("which").arg(term).output().map(|o| o.status.success()).unwrap_or(false) {
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

    format!("[X] Не найден терминал для {}. Установите {} или другой терминал.",
            de.name(), de.preferred_terminal())
}

/// Обновление системы
/// Запускаем в терминале, т.к. yay требует интерактивный ввод для sudo
pub fn update_system() -> String {
    run_in_terminal("yay -Syu", "Обновление системы")
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
