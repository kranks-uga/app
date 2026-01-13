use std::process::Command;

// Импортируем типы для работы с фоновыми задачами и диалогами
use crate::app::chat::{BackgroundTask, DialogType, TaskManager};

/*  ОСНОВНАЯ ФУНКЦИЯ: Обрабатывает текстовые команды от пользователя
 и переключает состояние интерфейса (открывает диалоги).
 
 # Аргументы
 `cmd` - текст команды от пользователя
 `dialog_type` - тип отображаемого диалога (изменяется по ссылке)
 `dialog_title` - заголовок диалога (изменяется по ссылке)
 `dialog_message` - сообщение в диалоге (изменяется по ссылке)
 `dialog_input` - поле ввода в диалоге (изменяется по ссылке)
 `dialog_package` - имя пакета для операций (изменяется по ссылке)
 `show_dialog` - флаг отображения диалога (изменяется по ссылке)
 `task_manager` - менеджер фоновых задач
 
 # Возвращает
 - `Some(String)` - текстовый ответ для чата при успешной обработке команды
 - `None` - если команда не распознана */
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
    // Команда для открытия окна поиска пакетов через диалог
    if cmd == "поиск пакетов" || cmd == "найти пакеты" {
        *dialog_type = DialogType::PackageSearch;
        *dialog_title = "Поиск пакетов".to_string();
        *dialog_message = "Введите название пакета для поиска:".to_string();
        dialog_input.clear(); // Очищаем поле ввода перед открытием диалога
        *show_dialog = true; // Показываем диалоговое окно
        
        Some("Открываю диалог для поиска пакетов...".to_string())
    } 
    // Команда установки пакета: "установить <имя_пакета>"
    else if cmd.starts_with("установить ") {
        // Извлекаем имя пакета из команды (убираем префикс "установить ")
        let package_name = cmd.trim_start_matches("установить ").trim();
        
        // Проверяем, что указано имя пакета
        if package_name.is_empty() {
            Some("Укажите имя пакета. Пример: 'установить firefox'".to_string())
        } else {
            // Настраиваем диалог подтверждения для установки
            *dialog_type = DialogType::Confirmation;
            *dialog_title = "Установка пакета".to_string();
            *dialog_message = format!("Установить '{}' через yay?\nПотребуется ввод пароля в системном окне.", package_name);
            *dialog_package = package_name.to_string(); // Сохраняем имя пакета для дальнейшего использования
            *show_dialog = true;
            
            Some(format!("Подготовка к установке '{}'...", package_name))
        }
    } 
    // Команда удаления пакета: "удалить <имя_пакета>"
    else if cmd.starts_with("удалить ") {
        // Извлекаем имя пакета из команды
        let package_name = cmd.trim_start_matches("удалить ").trim();
        
        if package_name.is_empty() {
            Some("Укажите имя пакета для удаления.".to_string())
        } else {
            // Настраиваем диалог подтверждения для удаления
            *dialog_type = DialogType::Confirmation;
            *dialog_title = "Удаление пакета".to_string();
            *dialog_message = format!("Удалить '{}' из системы?", package_name);
            *dialog_package = package_name.to_string();
            *show_dialog = true;
            
            Some(format!("Подготовка к удалению '{}'...", package_name))
        }
    } 
    // Команда обновления всей системы
    else if cmd == "обновить систему" || cmd == "обновление" {
        // Настраиваем диалог подтверждения для обновления системы
        *dialog_type = DialogType::Confirmation;
        *dialog_title = "Обновление системы".to_string();
        *dialog_message = "Выполнить полное обновление yay -Syu?".to_string();
        *show_dialog = true;
        
        Some("Подготовка к обновлению...".to_string())
    } 
    // Быстрый поиск пакетов напрямую из чата (без открытия диалога)
    else if cmd.starts_with("поиск ") {
        // Извлекаем поисковый запрос
        let query = cmd.trim_start_matches("поиск ").trim();
        
        // Запускаем фоновую задачу поиска пакетов
        task_manager.execute_task(BackgroundTask::SearchPackages(query.to_string()));
        
        Some(format!("Ищу пакеты по запросу '{}'...", query))
    } 
    // Неизвестная команда - возвращаем None
    else {
        None
    }
}

// --- ФУНКЦИИ ИСПОЛНЕНИЯ (Вызываются TaskManager в фоновом потоке) ---

/*  Выполняет поиск пакетов в репозиториях Arch Linux с помощью `yay -Ss`
 
# Аргументы
 - `query` - поисковый запрос (имя или описание пакета)
 
# Возвращает
`String` - результат поиска в текстовом формате или сообщение об ошибке
 
# Примечание
Использует системную утилиту `yay`, которая должна быть установлена в системе
Возвращает сырой вывод команды, который затем форматируется в интерфейсе */
pub fn search_packages(query: &str) -> String {
    // Выполняем команду yay -Ss <запрос> для поиска пакетов
    let output = Command::new("yay")
        .args(["-Ss", query])
        .output(); // Захватываем stdout и stderr

    // Обрабатываем результат выполнения команды
    match output {
        Ok(out) => {
            // Преобразуем байты вывода в UTF-8 строку
            let res = String::from_utf8_lossy(&out.stdout).to_string();
            
            // Проверяем, не пустой ли результат
            if res.trim().is_empty() { 
                "Ничего не найдено.".to_string() 
            } else { 
                res 
            }
        }
        Err(e) => format!("Ошибка выполнения yay: {}", e),
    }
}

/// Устанавливает пакет с использованием `yay -S` и прав суперпользователя через `pkexec`
/// 
/// # Аргументы
/// - `package` - имя пакета для установки
/// 
/// # Возвращает
/// - `String` - сообщение об успехе или ошибке установки
/// 
/// # Примечание
/// - `pkexec` запускает графическое окно ввода пароля (вместо sudo в терминале)
/// - Флаг `--noconfirm` автоматически подтверждает все запросы (не спрашивает подтверждения)
/// - Пакет должен существовать в репозиториях или AUR
pub fn install_package(package: &str) -> String {
    // Выполняем установку с правами суперпользователя через pkexec
    let status = Command::new("pkexec")
        .args(["yay", "-S", "--noconfirm", package])
        .status(); // Получаем только код возврата
    
    // Анализируем код возврата команды
    match status {
        Ok(s) if s.success() => format!("Успешно: пакет '{}' установлен.", package),
        _ => format!("Ошибка: не удалось установить '{}' (возможно, отмена или нет сети).", package),
    }
}

/// Удаляет пакет из системы с использованием `yay -R` и прав суперпользователя
/// 
/// # Аргументы
/// - `package` - имя пакета для удаления
/// 
/// # Возвращает
/// - `String` - сообщение об успехе или ошибке удаления
/// 
/// # Примечание
/// - Удаляет пакет и его зависимости, которые больше не нужны (опционально)
/// - Использует `--noconfirm` для автоматического подтверждения операции
pub fn remove_package(package: &str) -> String {
    // Выполняем удаление пакета через pkexec
    let status = Command::new("pkexec")
        .args(["yay", "-R", "--noconfirm", package])
        .status();
    
    match status {
        Ok(s) if s.success() => format!("Успешно: пакет '{}' удален.", package),
        _ => format!("Ошибка при удалении '{}'.", package),
    }
}

/// Выполняет полное обновление системы с помощью `yay -Syu`
/// 
/// # Возвращает
/// - `String` - сообщение об успехе или ошибке обновления
/// 
/// # Примечание
/// - Обновляет все пакеты из официальных репозиториев и AUR
/// - `-Sy` - синхронизирует базы данных и обновляет пакеты
/// - `-u` - обновляет установленные пакеты
/// - Может занять значительное время в зависимости от количества обновлений
pub fn update_system() -> String {
    // Выполняем полное обновление системы
    let status = Command::new("pkexec")
        .args(["yay", "-Syu", "--noconfirm"])
        .status();
    
    match status {
        Ok(s) if s.success() => "Система успешно обновлена!".to_string(),
        _ => "Ошибка в процессе обновления системы.".to_string(),
    }
}

/// Проверяет наличие утилиты `yay` в системе
/// 
/// # Возвращает
/// - `String` - сообщение о наличии или отсутствии yay
/// 
/// # Примечание
/// - Использует команду `which` для поиска исполняемого файла в PATH
/// - yay необходим для работы всех функций этого модуля
/// - В случае отсутствия yay, предлагает команду для его установки
pub fn check_yay_installed() -> String {
    // Ищем yay в системных путях
    let output = Command::new("which").arg("yay").output();
    
    // Проверяем успешность выполнения команды which
    if output.is_ok() && output.unwrap().status.success() {
        "yay найден и готов к работе.".to_string()
    } else {
        "ВНИМАНИЕ: yay не найден. Установите его: sudo pacman -S yay".to_string()
    }
}