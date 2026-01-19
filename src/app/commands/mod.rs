//! Обработка команд пользователя

pub mod base;
pub mod system;
pub mod package;
pub mod guide;

use super::chat::{DialogState, TaskManager};
use super::guides::GuideRegistry;

/// Обрабатывает команду и возвращает ответ
///
/// Возвращает `Some(response)` если команда распознана, `None` если нет
pub fn process_command(
    input: &str,
    assistant_name: &str,
    dialog: &mut DialogState,
    tasks: &TaskManager,
    guides: &GuideRegistry,
) -> Option<String> {
    let cmd = input.trim().to_lowercase();

    // 1. Базовые команды (время, дата, помощь)
    if let Some(r) = base::process_basic_command(&cmd, assistant_name) {
        return Some(r);
    }

    // 2. Системные команды (выключение, перезагрузка)
    if let Some(r) = system::process_system_command(&cmd) {
        return Some(r);
    }

    // 3. Пакетный менеджер
    if let Some(r) = package::process_package_command(&cmd, dialog, tasks) {
        return Some(r);
    }

    // 4. Гайды
    if let Some(r) = guide::process_guide_command(&cmd, guides) {
        return Some(r);
    }

    None
}
