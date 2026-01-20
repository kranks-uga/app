use chrono::Local;
use std::collections::HashMap;
use std::process::Command;

/// Тип функции-обработчика инструмента
pub type ToolHandler = fn() -> String;

/// Описание одного инструмента
pub struct Tool {
    pub name: String,
    pub description: String,
    pub handler: ToolHandler,
}

/// Контейнер для всех инструментов
pub struct ToolRegistry {
    tools: HashMap<String, Tool>,
}

impl ToolRegistry {
    /// Создаёт реестр с базовыми инструментами
    pub fn new() -> Self {
        let mut registry = Self {
            tools: HashMap::new(),
        };

        // Регистрируем базовые инструменты
        registry.register("время", "получить текущее время", || {
            Local::now().format("%H:%M:%S").to_string()
        });

        registry.register("дата", "получить текущую дату", || {
            Local::now().format("%d.%m.%Y").to_string()
        });

        registry.register("дата_и_время", "получить дату и время", || {
            Local::now().format("%d.%m.%Y %H:%M:%S").to_string()
        });

        registry.register("список_гайдов", "показать доступные обучающие гайды", || {
            "pacman, aur, wifi, systemd, gpu, audio, locale, backup".to_string()
        });

        // Системная информация
        registry.register("память", "показать использование RAM", || {
            get_memory_info()
        });

        registry.register("диск", "показать использование дисков", || {
            get_disk_info()
        });

        registry.register("cpu", "показать информацию о процессоре", || {
            get_cpu_info()
        });

        registry.register("система", "показать общую информацию о системе", || {
            format!(
                "Память: {}\nCPU: {}\nДиск: {}",
                get_memory_info(),
                get_cpu_info(),
                get_disk_info()
            )
        });

        registry
    }

    /// Регистрирует новый инструмент
    pub fn register(&mut self, name: &str, description: &str, handler: ToolHandler) {
        self.tools.insert(
            name.to_string(),
            Tool {
                name: name.to_string(),
                description: description.to_string(),
                handler,
            },
        );
    }

    /// Выполняет инструмент по имени
    pub fn execute(&self, name: &str) -> Option<String> {
        self.tools.get(name).map(|tool| (tool.handler)())
    }

    /// Генерирует системный промпт с описанием всех инструментов
    pub fn generate_system_prompt(&self) -> String {
        let mut tools_list = String::new();
        for tool in self.tools.values() {
            tools_list.push_str(&format!("- [TOOL:{}] - {}\n", tool.name, tool.description));
        }

        format!(
r#"Ты помощник Альфонс для Arch Linux. Отвечай кратко и по делу на русском языке.

ДОСТУПНЫЕ ИНСТРУМЕНТЫ:
{}
Формат использования: [TOOL:название]

ДОСТУПНЫЕ КОМАНДЫ (ты можешь выполнять их за пользователя):
Формат: [CMD:команда]

▸ Базовые:
  [CMD:очистить] - очистить чат
  [CMD:помощь] - показать справку

▸ Пакеты (yay/pacman):
  [CMD:поиск <запрос>] - найти пакеты
  [CMD:установить <пакет>] - запросить установку (откроется диалог подтверждения!)
  [CMD:удалить <пакет>] - запросить удаление (откроется диалог подтверждения!)
  [CMD:обновить систему] - запросить обновление (откроется диалог подтверждения!)

▸ Система:
  [CMD:выключить пк] - выключить компьютер
  [CMD:перезагрузить] - перезагрузить компьютер

▸ Гайды:
  [CMD:гайды] - показать список всех гайдов
  [CMD:гайд <тема>] - показать конкретный гайд
  Доступные темы: pacman, aur, wifi, systemd, gpu, audio, locale, backup

ВАЖНЫЕ ПРАВИЛА:
1. Используй [TOOL:...] для получения информации (время, дата)
2. Используй [CMD:...] для выполнения команд за пользователя
3. КОМАНДЫ установки/удаления/обновления ТОЛЬКО открывают диалог! НЕ говори "установлено" или "обновлено" сразу!
4. После команды установки скажи "откроется диалог подтверждения" или просто используй команду
5. Если спрашивают "как установить" - объясни или предложи [CMD:гайд pacman]
6. ОПАСНЫЕ КОМАНДЫ (выключить пк, перезагрузить) выполняй ТОЛЬКО если пользователь ЯВНО попросил это сделать!
7. На вопросы "что ты умеешь?" или "какие команды есть?" - ОТВЕЧАЙ ТЕКСТОМ, НЕ выполняй команды!

ПРИМЕРЫ:
- "Который час?" -> "Сейчас [TOOL:время]"
- "Установи firefox" -> "[CMD:установить firefox]" (НЕ говори "установлен"!)
- "Найди пакет vim" -> "[CMD:поиск vim]"
- "Как настроить wifi?" -> "[CMD:гайд wifi]"
- "Покажи гайды" -> "[CMD:гайды]"
- "Обнови систему" -> "[CMD:обновить систему]" (откроется диалог)
- "Что ты умеешь?" -> Перечисли возможности ТЕКСТОМ, НЕ выполняй команды!
- "Выключи компьютер" -> "[CMD:выключить пк]" (только по явному запросу!)

Отвечай кратко. НЕ пиши текст после команд установки/удаления/обновления."#,
            tools_list
        )
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Функции получения системной информации
// ============================================================================

/// Получает информацию об использовании памяти
fn get_memory_info() -> String {
    let output = Command::new("free")
        .args(["-h", "--si"])
        .output();

    match output {
        Ok(out) => {
            let text = String::from_utf8_lossy(&out.stdout);
            // Парсим вторую строку (Mem:)
            if let Some(line) = text.lines().nth(1) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    return format!("{} / {} (использовано)", parts[2], parts[1]);
                }
            }
            "Не удалось получить".into()
        }
        Err(_) => "Ошибка выполнения free".into(),
    }
}

/// Получает информацию об использовании дисков
fn get_disk_info() -> String {
    let output = Command::new("df")
        .args(["-h", "/"])
        .output();

    match output {
        Ok(out) => {
            let text = String::from_utf8_lossy(&out.stdout);
            if let Some(line) = text.lines().nth(1) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 5 {
                    return format!("{} / {} ({})", parts[2], parts[1], parts[4]);
                }
            }
            "Не удалось получить".into()
        }
        Err(_) => "Ошибка выполнения df".into(),
    }
}

/// Получает информацию о процессоре
fn get_cpu_info() -> String {
    // Имя процессора
    let name = std::fs::read_to_string("/proc/cpuinfo")
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|l| l.starts_with("model name"))
                .and_then(|l| l.split(':').nth(1))
                .map(|s| s.trim().to_string())
        })
        .unwrap_or_else(|| "Неизвестно".into());

    // Загрузка (из /proc/loadavg)
    let load = std::fs::read_to_string("/proc/loadavg")
        .ok()
        .and_then(|s| s.split_whitespace().next().map(|s| s.to_string()))
        .unwrap_or_else(|| "?".into());

    format!("{} (загрузка: {})", name, load)
}
