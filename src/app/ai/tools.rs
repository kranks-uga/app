use chrono::Local;
use std::collections::HashMap;

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
6. Будь проактивным - если можешь выполнить команду, делай это

ПРИМЕРЫ:
- "Который час?" -> "Сейчас [TOOL:время]"
- "Установи firefox" -> "[CMD:установить firefox]" (НЕ говори "установлен"!)
- "Найди пакет vim" -> "[CMD:поиск vim]"
- "Как настроить wifi?" -> "[CMD:гайд wifi]"
- "Покажи гайды" -> "[CMD:гайды]"
- "Обнови систему" -> "[CMD:обновить систему]" (откроется диалог)

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
