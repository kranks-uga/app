// Подмодули команд
pub mod base;
pub mod system;
pub mod package;

use super::chat::{BackgroundTask, DialogType, TaskManager};

/// Основная функция обработки команд
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
    let cmd = input.trim().to_lowercase();
    
    // Пробуем обработать базовые команды
    if let Some(response) = base::process_basic_command(&cmd, assistant_name) {
        return Some(response);
    }
    
    // Пробуем обработать системные команды
    if let Some(response) = system::process_system_command(&cmd) {
        return Some(response);
    }
    
    // Пробуем обработать команды управления пакетами
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
    
    // Если команда не распознана
    Some(format!("Неизвестная команда: '{}'", input.trim()))
}