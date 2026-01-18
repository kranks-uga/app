// Подключение внутренних модулей с логикой конкретных команд
pub mod base;    // Базовые: время, дата, приветствие
pub mod system;  // Системные: выключение, перезагрузка
pub mod package; // Пакетные: установка и удаление (pacman/yay)

use super::chat::{DialogType, TaskManager};

/// Главная точка входа для распределения команд по категориям.
/// Принимает ввод пользователя и ссылки на состояние интерфейса (диалоги, менеджер задач).
pub fn process_command(
    input: &str,
    assistant_name: &str,
    dialog_type: &mut DialogType,
    dialog_title: &mut String,
    dialog_message: &mut String,
    dialog_input: &mut String,
    dialog_package: &mut String,
    show_dialog: &mut bool,
    task_manager: &TaskManager,
) -> Option<String> {
    // Нормализация ввода: удаление пробелов и приведение к нижнему регистру
    let cmd = input.trim().to_lowercase();
    
    // 1. Попытка обработки простых текстовых ответов
    if let Some(response) = base::process_basic_command(&cmd, assistant_name) {
        return Some(response);
    }
    
    // 2. Попытка выполнения системных действий (shutdown/reboot)
    if let Some(response) = system::process_system_command(&cmd) {
        return Some(response);
    }
    
    // 3. Обработка команд пакетного менеджера (требует управления состоянием диалогов)
    if let Some(response) = package::process_package_command(
        &cmd,
        dialog_type,
        dialog_title,
        dialog_message,
        dialog_input,
        dialog_package,
        show_dialog,
        task_manager,
    ) {
        return Some(response);
    }
    
    // Возврат сообщения об ошибке, если ни один модуль не распознал команду
    None
}
