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

ИНСТРУМЕНТЫ (используй ТОЛЬКО когда пользователь явно спрашивает):
{}
Формат: [TOOL:название]. Используй инструмент времени/даты ТОЛЬКО если пользователь спросил "который час", "сколько времени", "какая дата" и т.п.

ГАЙДЫ: pacman, aur, wifi, systemd, gpu, audio, locale, backup
Если спрашивают как что-то сделать в Linux, рекомендуй: "гайд <тема>"

Примеры:
- "Который час?" -> "Сейчас [TOOL:время]"
- "Как установить пакет?" -> "sudo pacman -S <пакет>. Подробнее: гайд pacman"
- "Привет" -> "Привет! Чем могу помочь?"

НЕ используй инструменты без явного запроса. Отвечай просто и по делу."#,
            tools_list
        )
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}
