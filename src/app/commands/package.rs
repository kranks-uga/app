use std::process::Command;
// Импортируем типы для работы с фоновыми задачами и диалогами
use crate::app::chat::{BackgroundTask, DialogType, TaskManager};

/// ОСНОВНАЯ ФУНКЦИЯ: Обрабатывает текстовые команды от пользователя
/// и переключает состояние интерфейса (открывает диалоги).
pub fn process_package_command(
    cmd: &str,
    dialog_type: &mut DialogType,
    dialog_title: &mut String,
    dialog_message: &mut String,
    dialog_input: &mut String,
    dialog_package: &mut String,
    show_dialog: &mut bool,
    task_manager: &TaskManager,
) -> Option<String> {
    // Команда для открытия окна поиска
    if cmd == "поиск пакетов" || cmd == "найти пакеты" {
        *dialog_type = DialogType::PackageSearch;
        *dialog_title = "Поиск пакетов".to_string();
        *dialog_message = "Введите название пакета для поиска:".to_string();
        dialog_input.clear();
        *show_dialog = true;
        Some("Открываю диалог для поиска пакетов...".to_string())
    } 
    // Команда "установить <имя>"
    else if cmd.starts_with("установить ") {
        let package_name = cmd.trim_start_matches("установить ").trim();
        if package_name.is_empty() {
            Some("Укажите имя пакета. Пример: 'установить firefox'".to_string())
        } else {
            *dialog_type = DialogType::Confirmation;
            *dialog_title = "Установка пакета".to_string();
            *dialog_message = format!("Установить '{}' через yay?\nПотребуется ввод пароля в системном окне.", package_name);
            *dialog_package = package_name.to_string();
            *show_dialog = true;
            Some(format!("Подготовка к установке '{}'...", package_name))
        }
    } 
    // Команда "удалить <имя>"
    else if cmd.starts_with("удалить ") {
        let package_name = cmd.trim_start_matches("удалить ").trim();
        if package_name.is_empty() {
            Some("Укажите имя пакета для удаления.".to_string())
        } else {
            *dialog_type = DialogType::Confirmation;
            *dialog_title = "Удаление пакета".to_string();
            *dialog_message = format!("Удалить '{}' из системы?", package_name);
            *dialog_package = package_name.to_string();
            *show_dialog = true;
            Some(format!("Подготовка к удалению '{}'...", package_name))
        }
    } 
    // Команда обновления системы
    else if cmd == "обновить систему" || cmd == "обновление" {
        *dialog_type = DialogType::Confirmation;
        *dialog_title = "Обновление системы".to_string();
        *dialog_message = "Выполнить полное обновление yay -Syu?".to_string();
        *show_dialog = true;
        Some("Подготовка к обновлению...".to_string())
    } 
    // Быстрый поиск через чат (команда "поиск <текст>")
    else if cmd.starts_with("поиск ") {
        let query = cmd.trim_start_matches("поиск ").trim();
        task_manager.execute_task(BackgroundTask::SearchPackages(query.to_string()));
        Some(format!("Ищу пакеты по запросу '{}'...", query))
    } 
    else {
        None
    }
}

// --- ФУНКЦИИ ИСПОЛНЕНИЯ (Вызываются TaskManager в фоновом потоке) ---

/// Поиск пакетов: использует 'yay -Ss' и возвращает результат строкой
pub fn search_packages(query: &str) -> String {
    let output = Command::new("yay")
        .args(["-Ss", query])
        .output();

    match output {
        Ok(out) => {
            let res = String::from_utf8_lossy(&out.stdout).to_string();
            if res.trim().is_empty() { "Ничего не найдено.".to_string() } else { res }
        }
        Err(e) => format!("Ошибка выполнения yay: {}", e),
    }
}

/// Установка пакета: использует 'pkexec' для вызова GUI-окна пароля
/// Флаг --noconfirm нужен, чтобы yay не задавал вопросов в фоне
pub fn install_package(package: &str) -> String {
    let status = Command::new("pkexec")
        .args(["yay", "-S", "--noconfirm", package])
        .status();
    
    match status {
        Ok(s) if s.success() => format!("Успешно: пакет '{}' установлен.", package),
        _ => format!("Ошибка: не удалось установить '{}' (возможно, отмена или нет сети).", package),
    }
}

/// Удаление пакета
pub fn remove_package(package: &str) -> String {
    let status = Command::new("pkexec")
        .args(["yay", "-R", "--noconfirm", package])
        .status();
    
    match status {
        Ok(s) if s.success() => format!("Успешно: пакет '{}' удален.", package),
        _ => format!("Ошибка при удалении '{}'.", package),
    }
}

/// Обновление системы
pub fn update_system() -> String {
    let status = Command::new("pkexec")
        .args(["yay", "-Syu", "--noconfirm"])
        .status();
    
    match status {
        Ok(s) if s.success() => "Система успешно обновлена!".to_string(),
        _ => "Ошибка в процессе обновления системы.".to_string(),
    }
}

/// Проверка наличия утилиты yay в системе
pub fn check_yay_installed() -> String {
    let output = Command::new("which").arg("yay").output();
    if output.is_ok() && output.unwrap().status.success() {
        "yay найден и готов к работе.".to_string()
    } else {
        "ВНИМАНИЕ: yay не найден. Установите его: sudo pacman -S yay".to_string()
    }
}
