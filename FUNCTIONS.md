# Документация функций Альфонс

Полное описание всех публичных функций, структур и модулей проекта с примерами использования.

## Содержание

- [Модуль assistant_app](#модуль-assistant_app)
- [Модуль chat](#модуль-chat)
- [Модуль config](#модуль-config)
- [Модуль ai](#модуль-ai)
- [Модуль commands](#модуль-commands)
- [Модуль guides](#модуль-guides)
- [Модуль ui](#модуль-ui)
- [Модуль desktop](#модуль-desktop)
- [Модуль installer](#модуль-installer)
- [Модуль constants](#модуль-constants)

---

## Модуль assistant_app

**Файл:** `src/app/assistant_app.rs`

Главная структура приложения, объединяющая все компоненты.

### Структура `AssistantApp`

```rust
pub struct AssistantApp {
    pub config: Config,
    pub chat: ChatHistory,
    pub guides: GuideRegistry,
    pub ai: Arc<LocalAi>,
    pub input_text: String,
    pub show_settings: bool,
    pub dialog: DialogState,
    pub input_history: InputHistory,
    pub ollama_online: Arc<AtomicBool>,
    pub ollama_installed: Arc<AtomicBool>,
    pub yay_installed: Arc<AtomicBool>,
    pub custom_model_exists: Arc<AtomicBool>,
    pub app_installed: Arc<AtomicBool>,
    pub desktop_env: DesktopEnvironment,
    pub de_styles: DeStyles,
    pub tasks: TaskManager,
}
```

### Функции

#### `AssistantApp::new(cc: &CreationContext) -> Self`

Создаёт новый экземпляр приложения.

**Пример из проекта (src/app/assistant_app.rs:56-125):**
```rust
pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
    let (tasks, task_receiver) = TaskManager::new();
    let config = Config::load();

    // Определяем окружение рабочего стола
    let desktop_env = DesktopEnvironment::detect();
    let de_styles = DeStyles::for_de(desktop_env);

    let mut chat = ChatHistory::default();
    chat.add_message(&config.assistant_name, messages::WELCOME);

    let ai = Arc::new(LocalAi::new());
    ai.set_model(&config.ollama_model);

    // Запускаем проверку статуса Ollama в фоне
    let ollama_online = Arc::new(AtomicBool::new(false));
    let ollama_online_clone = ollama_online.clone();
    tokio::spawn(async move {
        let status = super::ai::local_provider::check_ollama_status().await;
        ollama_online_clone.store(status, Ordering::SeqCst);
    });
    // ...
}
```

---

#### `process_input(&mut self)`

Обрабатывает ввод пользователя.

**Пример из проекта (src/app/assistant_app.rs:140-172):**
```rust
pub fn process_input(&mut self) {
    let input = self.input_text.trim();
    if input.is_empty() {
        return;
    }

    let input = input.to_string();
    self.input_history.push(&input);
    self.chat.add_message("Вы", &input);

    // Пробуем обработать как команду
    let response = commands::process_command(
        &input,
        &self.config.assistant_name,
        &mut self.dialog,
        &self.tasks,
        &self.guides,
    );

    if let Some(text) = response {
        if text == CMD_CLEAR_CHAT {
            self.clear_chat();
        } else {
            self.chat.add_message(&self.config.assistant_name, text);
        }
    } else {
        // Отправляем в AI
        self.send_to_ai(&input);
    }

    self.input_text.clear();
}
```

---

#### `send_to_ai(&self, input: &str)`

Отправляет запрос в AI асинхронно.

**Пример из проекта (src/app/assistant_app.rs:175-188):**
```rust
fn send_to_ai(&self, input: &str) {
    let ai = Arc::clone(&self.ai);
    let tx = self.tasks.result_sender.clone();
    let name = self.config.assistant_name.clone();
    let input = input.to_string();

    tokio::spawn(async move {
        let response = match ai.generate(&input).await {
            Ok(text) => format!("{}: {}", name, text),
            Err(e) => format!("Ошибка ИИ: {}", e),
        };
        let _ = tx.send(response);
    });
}
```

---

#### `process_ai_commands(&mut self, text: &str) -> String`

Обрабатывает маркеры `[CMD:...]` в ответе AI.

**Пример из проекта (src/app/assistant_app.rs:209-248):**
```rust
fn process_ai_commands(&mut self, text: &str) -> String {
    let cmd_re = cmd_regex();
    let mut result = text.to_string();

    // Находим все команды в тексте
    let commands: Vec<String> = cmd_re
        .captures_iter(text)
        .map(|cap| cap[1].to_string())
        .collect();

    // Выполняем каждую команду
    for cmd in commands {
        let marker = format!("[CMD:{}]", cmd);

        let cmd_response = commands::process_command(
            &cmd,
            &self.config.assistant_name,
            &mut self.dialog,
            &self.tasks,
            &self.guides,
        );

        if let Some(response) = cmd_response {
            if response == commands::base::CMD_CLEAR_CHAT {
                self.clear_chat();
                result = result.replace(&marker, "");
            } else {
                result = result.replace(&marker, "");
            }
        } else {
            result = result.replace(&marker, &format!("[!] команда '{}' не распознана", cmd));
        }
    }

    result
}
```

**Пример использования:**
```
Вход: "Сейчас [CMD:время], погода хорошая"
Выход: "Сейчас , погода хорошая" (команда "время" выполнится отдельно)
```

---

## Модуль chat

**Файл:** `src/app/chat.rs`

### Перечисление `DialogType`

```rust
pub enum DialogType {
    Info,           // Информационное окно
    PackageSearch,  // Поиск пакетов
    Confirmation,   // Подтверждение действия
}
```

### Структура `DialogState`

**Пример из проекта (src/app/chat.rs:26-34):**
```rust
pub struct DialogState {
    pub visible: bool,
    pub dialog_type: DialogType,
    pub title: String,
    pub message: String,
    pub input: String,
    pub package: String,
}
```

#### `show_confirm(&mut self, title: &str, message: &str, package: &str)`

**Пример из проекта (src/app/chat.rs:50-56):**
```rust
pub fn show_confirm(&mut self, title: &str, message: &str, package: &str) {
    self.visible = true;
    self.dialog_type = DialogType::Confirmation;
    self.title = title.to_string();
    self.message = message.to_string();
    self.package = package.to_string();
}
```

**Пример вызова (src/app/commands/package.rs:26-31):**
```rust
dialog.show_confirm(
    "Установка пакета",
    &format!("Установить '{}' через yay?", package),
    package,
);
```

---

### Перечисление `BackgroundTask`

**Пример из проекта (src/app/chat.rs:71-85):**
```rust
pub enum BackgroundTask {
    SearchPackages(String),
    InstallPackage(String),
    RemovePackage(String),
    UpdateSystem,
    InstallYay,
    ShutdownSystem,
    RebootSystem,
    CreateCustomModel,
    InstallToSystem,
    UninstallFromSystem,
    InstallOllama,
    StartOllama,
}
```

---

### Структура `ChatHistory`

#### `add_message(&mut self, sender: impl Into<String>, text: impl Into<String>)`

**Пример из проекта (src/app/chat.rs:114-125):**
```rust
pub fn add_message(&mut self, sender: impl Into<String>, text: impl Into<String>) {
    self.messages.push_back(ChatMessage {
        sender: sender.into(),
        text: text.into(),
        timestamp: Local::now(),
    });

    // Удаляем старые сообщения при превышении лимита
    if self.messages.len() > self.max_messages {
        self.messages.pop_front();
    }
}
```

**Пример вызова:**
```rust
app.chat.add_message("Система", messages::OLLAMA_INSTALLING);
app.chat.add_message(&config.assistant_name, "Привет!");
app.chat.add_message("Вы", &input);
```

---

### Структура `TaskManager`

**Пример из проекта (src/app/chat.rs:156-212):**
```rust
pub fn new() -> (Self, Receiver<String>) {
    let (task_sender, task_receiver) = mpsc::channel::<BackgroundTask>();
    let (result_sender, result_receiver) = mpsc::channel::<String>();

    let result_sender_clone = result_sender.clone();
    let is_processing = Arc::new(AtomicBool::new(false));
    let is_processing_clone = is_processing.clone();

    // Фоновый поток для обработки задач
    thread::spawn(move || {
        while let Ok(task) = task_receiver.recv() {
            let result = match task {
                BackgroundTask::SearchPackages(query) => {
                    super::commands::package::search_packages(&query)
                }
                BackgroundTask::InstallPackage(package) => {
                    super::commands::package::install_package(&package)
                }
                BackgroundTask::UpdateSystem => super::commands::package::update_system(),
                BackgroundTask::ShutdownSystem => super::commands::system::execute_shutdown(),
                // ...
            };
            let _ = result_sender_clone.send(result);
            is_processing_clone.store(false, Ordering::SeqCst);
        }
    });
    // ...
}
```

**Пример вызова:**
```rust
app.tasks.execute(BackgroundTask::SearchPackages("firefox".into()));
app.tasks.execute(BackgroundTask::InstallPackage("vim".into()));
app.tasks.execute(BackgroundTask::UpdateSystem);
```

---

### Структура `InputHistory`

**Пример из проекта (src/app/chat.rs:252-266):**
```rust
pub fn push(&mut self, input: &str) {
    let input = input.trim();
    if input.is_empty() {
        return;
    }
    // Не добавляем дубликаты подряд
    if self.entries.last().map(|s| s.as_str()) != Some(input) {
        self.entries.push(input.to_string());
        if self.entries.len() > MAX_INPUT_HISTORY {
            self.entries.remove(0);
        }
    }
    self.position = None;
}
```

**Пример навигации (src/app/chat.rs:269-286):**
```rust
pub fn up(&mut self, current: &str) -> Option<&str> {
    if self.entries.is_empty() {
        return None;
    }

    match self.position {
        None => {
            self.current_input = current.to_string();
            self.position = Some(self.entries.len() - 1);
        }
        Some(0) => return Some(&self.entries[0]),
        Some(pos) => {
            self.position = Some(pos - 1);
        }
    }

    self.position.map(|p| self.entries[p].as_str())
}
```

---

## Модуль config

**Файл:** `src/app/config.rs`

### Структура `Config`

**Пример из проекта (src/app/config.rs:9-15):**
```rust
#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub assistant_name: String,
    pub accent_color: [u8; 3],
    #[serde(default = "default_ollama_model")]
    pub ollama_model: String,
}
```

**Пример JSON конфигурации:**
```json
{
  "assistant_name": "Альфонс",
  "accent_color": [61, 174, 233],
  "ollama_model": "alfons"
}
```

#### `Config::load() -> Self`

**Пример из проекта (src/app/config.rs:33-35):**
```rust
pub fn load() -> Self {
    confy::load(CONFIG_APP_NAME, "config").unwrap_or_default()
}
```

#### `Config::save(&self) -> Result<(), String>`

**Пример из проекта (src/app/config.rs:38-41):**
```rust
pub fn save(&self) -> Result<(), String> {
    confy::store(CONFIG_APP_NAME, "config", self)
        .map_err(|e| format!("Не удалось сохранить настройки: {}", e))
}
```

**Пример вызова (src/app/ui/mod.rs:446-448):**
```rust
if changed {
    if let Err(e) = app.config.save() {
        app.chat.add_message("Система", &e);
    }
}
```

---

## Модуль ai

**Файл:** `src/app/ai/`

### Модуль local_provider

### Структура `LocalAi`

**Пример из проекта (src/app/ai/local_provider.rs:35-39):**
```rust
pub struct LocalAi {
    client: Client,
    model: RwLock<String>,
    tools: ToolRegistry,
}
```

#### `LocalAi::generate(&self, input: &str) -> Result<String, String>`

**Пример из проекта (src/app/ai/local_provider.rs:66-89):**
```rust
pub async fn generate(&self, input: &str) -> Result<String, String> {
    let payload = OllamaRequest {
        model: self.get_model(),
        prompt: input.to_string(),
        stream: false,
        system: self.tools.generate_system_prompt(),
    };

    let response = self
        .client
        .post(OLLAMA_URL)
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("{}: {}", errors::OLLAMA_CONNECTION, e))?;

    let data: OllamaResponse = response
        .json()
        .await
        .map_err(|e| format!("{}: {}", errors::OLLAMA_PARSE, e))?;

    // Обрабатываем инструменты в ответе
    Ok(self.process_response(&data.response))
}
```

#### `process_response(&self, response: &str) -> String`

Обрабатывает маркеры `[TOOL:...]` в ответе AI.

**Пример из проекта (src/app/ai/local_provider.rs:92-104):**
```rust
fn process_response(&self, response: &str) -> String {
    let tool_re = tool_regex();
    let with_tools = tool_re.replace_all(response, |caps: &regex::Captures| {
        let tool = &caps[1];
        self.tools
            .execute(tool)
            .unwrap_or_else(|| format!("[?{}]", tool))
    });

    with_tools.to_string()
}
```

**Пример преобразования:**
```
Вход: "Сейчас [TOOL:время], дата: [TOOL:дата]"
Выход: "Сейчас 14:30:25, дата: 29.01.2026"
```

---

#### `check_ollama_status() -> bool`

**Пример из проекта (src/app/ai/local_provider.rs:114-126):**
```rust
pub async fn check_ollama_status() -> bool {
    let client = Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .unwrap_or_default();

    client
        .get("http://localhost:11434/api/tags")
        .send()
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false)
}
```

---

#### `create_custom_model() -> String`

**Пример из проекта (src/app/ai/local_provider.rs:148-207):**
```rust
pub fn create_custom_model() -> String {
    // Проверяем, что базовая модель существует
    if !is_base_model_exists() {
        return errors::MODEL_BASE_NOT_FOUND.to_string();
    }

    // Проверяем, не существует ли уже модель
    if is_custom_model_exists() {
        return messages::MODEL_EXISTS.to_string();
    }

    // Находим путь к Modelfile
    let modelfile_paths = [
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.join("Modelfile")))
            .unwrap_or_default(),
        std::path::PathBuf::from("Modelfile"),
        dirs::config_dir()
            .map(|p| p.join("alfons-assistant").join("Modelfile"))
            .unwrap_or_default(),
    ];

    // Создаём модель
    match Command::new("ollama")
        .args(["create", OLLAMA_CUSTOM_MODEL, "-f"])
        .arg(&modelfile)
        .output()
    {
        Ok(output) if output.status.success() => messages::MODEL_CREATED.to_string(),
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            format!("{} ({})", errors::MODEL_CREATE_FAILED, stderr.trim())
        }
        Err(e) => format!("{} ({})", errors::MODEL_CREATE_FAILED, e),
    }
}
```

---

### Модуль tools

### Структура `ToolRegistry`

**Пример регистрации инструментов (src/app/ai/tools.rs:22-73):**
```rust
pub fn new() -> Self {
    let mut registry = Self {
        tools: HashMap::new(),
    };

    // Регистрируем базовые инструменты
    registry.register(
        "время",
        "получить текущее время",
        || Local::now().format("%H:%M:%S").to_string(),
    );

    registry.register(
        "дата",
        "получить текущую дату",
        || Local::now().format("%d.%m.%Y").to_string(),
    );

    registry.register(
        "память",
        "показать использование RAM",
        get_memory_info,
    );

    registry.register(
        "система",
        "показать общую информацию о системе",
        || {
            format!(
                "Память: {}\nCPU: {}\nДиск: {}",
                get_memory_info(),
                get_cpu_info(),
                get_disk_info()
            )
        },
    );

    registry
}
```

#### `generate_system_prompt(&self) -> String`

**Пример из проекта (src/app/ai/tools.rs:93-150):**
```rust
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
  [CMD:установить <пакет>] - запросить установку
  [CMD:удалить <пакет>] - запросить удаление

ПРИМЕРЫ:
- "Который час?" -> "Сейчас [TOOL:время]"
- "Установи firefox" -> "[CMD:установить firefox]"
- "Найди пакет vim" -> "[CMD:поиск vim]"
"#,
        tools_list
    )
}
```

---

#### `get_memory_info() -> String`

**Пример из проекта (src/app/ai/tools.rs:164-181):**
```rust
fn get_memory_info() -> String {
    let output = Command::new("free").args(["-h", "--si"]).output();

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
```

**Пример вывода:** `"4.2G / 16G (использовано)"`

---

#### `get_cpu_info() -> String`

**Пример из проекта (src/app/ai/tools.rs:203-222):**
```rust
fn get_cpu_info() -> String {
    // Имя процессора из /proc/cpuinfo
    let name = std::fs::read_to_string("/proc/cpuinfo")
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|l| l.starts_with("model name"))
                .and_then(|l| l.split(':').nth(1))
                .map(|s| s.trim().to_string())
        })
        .unwrap_or_else(|| "Неизвестно".into());

    // Загрузка из /proc/loadavg
    let load = std::fs::read_to_string("/proc/loadavg")
        .ok()
        .and_then(|s| s.split_whitespace().next().map(|s| s.to_string()))
        .unwrap_or_else(|| "?".into());

    format!("{} (загрузка: {})", name, load)
}
```

**Пример вывода:** `"AMD Ryzen 5 5600X 6-Core Processor (загрузка: 1.23)"`

---

## Модуль commands

**Файл:** `src/app/commands/`

### Главная функция process_command

**Пример из проекта (src/app/commands/mod.rs:15-49):**
```rust
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
        command_log::log_command(&cmd, &r);
        return Some(r);
    }

    // 2. Системные команды (выключение, перезагрузка)
    if let Some(r) = system::process_system_command(&cmd, dialog) {
        command_log::log_command(&cmd, &r);
        return Some(r);
    }

    // 3. Пакетный менеджер
    if let Some(r) = package::process_package_command(&cmd, dialog, tasks) {
        command_log::log_command(&cmd, &r);
        return Some(r);
    }

    // 4. Гайды
    if let Some(r) = guide::process_guide_command(&cmd, guides) {
        command_log::log_command(&cmd, "гайд показан");
        return Some(r);
    }

    None
}
```

---

### Модуль base

**Пример из проекта (src/app/commands/base.rs:7-52):**
```rust
pub fn process_basic_command(cmd: &str, assistant_name: &str) -> Option<String> {
    match cmd {
        // Приветствие
        "привет" | "здравствуй" | "хай" | "hello" => Some(format!(
            "Привет! Я {}, твой помощник для Arch Linux.",
            assistant_name
        )),

        // Очистка чата
        "очистить" | "очистить чат" | "clear" => {
            Some(CMD_CLEAR_CHAT.to_string())
        }

        // Повторение фразы
        cmd if cmd.starts_with("скажи ") => {
            let message = cmd.trim_start_matches("скажи ").trim();
            if message.is_empty() {
                Some("Что именно сказать?".to_string())
            } else {
                Some(message.to_string())
            }
        }

        // Время
        "время" | "который час" | "time" => Some(format!(
            "Текущее время: {}",
            Local::now().format("%H:%M:%S")
        )),

        // Дата
        "дата" | "какое сегодня число" | "date" => {
            Some(format!("Сегодня: {}", Local::now().format("%d.%m.%Y")))
        }

        // Помощь
        "помощь" | "help" | "?" => Some(HELP_TEXT.to_string()),

        _ => None,
    }
}
```

**Текст справки (src/app/commands/base.rs:55-79):**
```rust
const HELP_TEXT: &str = "\
📋 Доступные команды:

▸ Базовые:
  время, дата, дата и время

▸ Пакеты (через yay):
  поиск <запрос>
  установить <пакет>
  удалить <пакет>
  обновить систему

▸ Система:
  выключить пк
  перезагрузить

▸ Гайды:
  гайды — список всех гайдов
  гайд <тема> — показать гайд

▸ Прочее:
  очистить — очистить чат
  помощь — эта справка

💡 Или просто задайте вопрос — ИИ постарается помочь!";
```

---

### Модуль package

**Пример из проекта (src/app/commands/package.rs:9-72):**
```rust
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
        dialog.show_confirm(
            "Удаление пакета",
            &format!("Удалить '{}' из системы?", package),
            package,
        );
        return Some(format!("Подготовка к удалению '{}'...", package));
    }

    // Обновление системы
    if cmd == "обновить систему" || cmd == "обновить" {
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
```

---

#### `run_in_terminal(cmd: &str, action: &str) -> String`

**Пример из проекта (src/app/commands/package.rs:156-189):**
```rust
fn run_in_terminal(cmd: &str, action: &str) -> String {
    let de = DesktopEnvironment::detect();
    let terminals = de.terminal_priority();

    for term in terminals {
        // Проверяем, установлен ли терминал
        if !Command::new("which")
            .arg(term)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
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

    format!(
        "[X] Не найден терминал для {}. Установите {}.",
        de.name(),
        de.preferred_terminal()
    )
}
```

---

#### `install_yay() -> String`

**Пример из проекта (src/app/commands/package.rs:207-254):**
```rust
pub fn install_yay() -> String {
    if is_yay_installed() {
        return messages::YAY_ALREADY.into();
    }

    // 1. Установка зависимостей
    let deps = Command::new("pkexec")
        .args([
            "pacman", "-S", "--needed", "--noconfirm",
            "git", "base-devel",
        ])
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
        .args([
            "-c",
            &format!("cd {} && makepkg -si --noconfirm", YAY_INSTALL_DIR),
        ])
        .status();

    // Очистка
    let _ = Command::new("rm").args(["-rf", YAY_INSTALL_DIR]).status();

    match build {
        Ok(s) if s.success() && is_yay_installed() => messages::YAY_INSTALLED.into(),
        _ => errors::YAY_BUILD_FAILED.into(),
    }
}
```

---

### Модуль system

**Пример из проекта (src/app/commands/system.rs:7-27):**
```rust
pub fn process_system_command(cmd: &str, dialog: &mut DialogState) -> Option<String> {
    match cmd {
        "выключить пк" | "выключить компьютер" => {
            dialog.show_confirm(
                "Выключение компьютера",
                "Вы уверены, что хотите выключить компьютер?",
                "__shutdown__",
            );
            Some("Подтвердите выключение...".into())
        }
        "перезагрузить" | "рестарт" => {
            dialog.show_confirm(
                "Перезагрузка",
                "Вы уверены, что хотите перезагрузить компьютер?",
                "__reboot__",
            );
            Some("Подтвердите перезагрузку...".into())
        }
        _ => None,
    }
}
```

---

### Модуль guide

**Пример из проекта (src/app/commands/guide.rs:4-65):**
```rust
pub fn process_guide_command(cmd: &str, guides: &GuideRegistry) -> Option<String> {
    // Список всех гайдов
    if cmd == "гайды" || cmd == "guides" || cmd == "обучение" {
        return Some(guides.format_list());
    }

    // Показать конкретный гайд: "гайд pacman"
    if cmd.starts_with("гайд ") || cmd.starts_with("guide ") {
        let guide_id = cmd
            .trim_start_matches("гайд ")
            .trim_start_matches("guide ")
            .trim();

        if let Some(guide) = guides.get(guide_id) {
            return Some(guide.format());
        }

        // Поиск по ключевому слову
        let results = guides.search(guide_id);
        if results.is_empty() {
            return Some(format!(
                "Гайд '{}' не найден.\n\nИспользуйте 'гайды' для списка.",
                guide_id
            ));
        } else if results.len() == 1 {
            return Some(results[0].format());
        } else {
            let mut output = format!(
                "Найдено {} гайдов по запросу '{}':\n\n",
                results.len(), guide_id
            );
            for guide in results {
                output.push_str(&format!("• {} — {}\n", guide.id, guide.title));
            }
            output.push_str("\nУточните запрос: гайд <название>");
            return Some(output);
        }
    }

    None
}
```

---

## Модуль guides

**Файл:** `src/app/guides/mod.rs`

### Структура `GuideStep`

**Пример из проекта (src/app/guides/mod.rs:4-29):**
```rust
pub struct GuideStep {
    pub instruction: String,
    pub command: Option<String>,
    pub note: Option<String>,
}

impl GuideStep {
    pub fn new(instruction: &str) -> Self {
        Self {
            instruction: instruction.to_string(),
            command: None,
            note: None,
        }
    }

    pub fn with_command(mut self, cmd: &str) -> Self {
        self.command = Some(cmd.to_string());
        self
    }

    pub fn with_note(mut self, note: &str) -> Self {
        self.note = Some(note.to_string());
        self
    }
}
```

---

### Структура `Guide`

#### `Guide::format(&self) -> String`

**Пример из проекта (src/app/guides/mod.rs:63-80):**
```rust
pub fn format(&self) -> String {
    let mut output = format!(" {}\n{}\n\n", self.title, self.description);

    for (i, step) in self.steps.iter().enumerate() {
        output.push_str(&format!("{}. {}\n", i + 1, step.instruction));

        if let Some(cmd) = &step.command {
            output.push_str(&format!("   $ {}\n", cmd));
        }

        if let Some(note) = &step.note {
            output.push_str(&format!("   ℹ {}\n", note));
        }
        output.push('\n');
    }

    output
}
```

**Пример вывода:**
```
 Основы Pacman
Базовые команды пакетного менеджера Arch Linux

1. Обновить список пакетов и систему
   $ sudo pacman -Syu
   ℹ Рекомендуется делать перед установкой новых пакетов

2. Установить пакет
   $ sudo pacman -S <пакет>
```

---

### Пример регистрации гайда

**Пример из проекта (src/app/guides/mod.rs:141-168):**
```rust
// Pacman
self.register(
    Guide::new(
        "pacman",
        "Основы Pacman",
        "Базовые команды пакетного менеджера Arch Linux",
    )
    .add_tags(&["пакеты", "установка", "обновление", "packages"])
    .add_step(
        GuideStep::new("Обновить список пакетов и систему")
            .with_command("sudo pacman -Syu")
            .with_note("Рекомендуется делать перед установкой новых пакетов"),
    )
    .add_step(GuideStep::new("Установить пакет").with_command("sudo pacman -S <пакет>"))
    .add_step(GuideStep::new("Удалить пакет").with_command("sudo pacman -R <пакет>"))
    .add_step(
        GuideStep::new("Удалить пакет с зависимостями")
            .with_command("sudo pacman -Rns <пакет>")
            .with_note("Удаляет также неиспользуемые зависимости и конфиги"),
    )
    .add_step(GuideStep::new("Поиск пакета").with_command("pacman -Ss <запрос>"))
    .add_step(GuideStep::new("Информация о пакете").with_command("pacman -Si <пакет>"))
    .add_step(GuideStep::new("Список установленных пакетов").with_command("pacman -Q"))
    .add_step(
        GuideStep::new("Очистить кэш пакетов")
            .with_command("sudo pacman -Sc")
            .with_note("Удаляет старые версии из /var/cache/pacman/pkg"),
    ),
);
```

---

## Модуль ui

**Файл:** `src/app/ui/`

### Модуль widgets

#### `render_message(ui, msg, accent)`

**Пример из проекта (src/app/ui/widgets.rs:7-92):**
```rust
pub fn render_message(ui: &mut egui::Ui, msg: &ChatMessage, accent: egui::Color32) {
    let is_user = msg.sender == "Вы";

    // Цвета
    let (bg, border, name_color) = if is_user {
        (
            egui::Color32::from_rgb(40, 80, 120),
            egui::Color32::from_rgb(60, 120, 180),
            egui::Color32::LIGHT_BLUE,
        )
    } else {
        (
            egui::Color32::from_gray(40),
            accent.gamma_multiply(0.3),
            accent,
        )
    };

    // Скругления (разные для пользователя и ассистента)
    let rounding = egui::Rounding {
        nw: 15.0,
        ne: 15.0,
        sw: if is_user { 15.0 } else { 2.0 },
        se: if is_user { 2.0 } else { 15.0 },
    };

    // Выравнивание
    let layout = if is_user {
        egui::Layout::right_to_left(egui::Align::TOP)
    } else {
        egui::Layout::left_to_right(egui::Align::TOP)
    };

    // Максимальная ширина пузыря - 70%
    let max_bubble_width = ui.available_width() * 0.7;

    ui.with_layout(layout, |ui| {
        egui::Frame::none()
            .fill(bg)
            .stroke(egui::Stroke::new(1.0, border))
            .rounding(rounding)
            .inner_margin(12.0)
            .show(ui, |ui| {
                ui.set_max_width(max_bubble_width);

                // Заголовок: имя + время
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(&msg.sender).strong().color(name_color));
                    ui.label(egui::RichText::new(" · ").weak());
                    ui.label(egui::RichText::new(
                        msg.timestamp.format("%H:%M").to_string()
                    ).color(egui::Color32::GRAY));
                });

                // Текст с копированием по клику
                let text_response = ui.add(
                    egui::Label::new(egui::RichText::new(&msg.text).color(egui::Color32::WHITE))
                        .wrap(true)
                        .sense(egui::Sense::click()),
                );

                if text_response.clicked() {
                    ui.output_mut(|o| o.copied_text = msg.text.clone());
                }
                text_response.on_hover_text("Нажмите чтобы скопировать");
            });
    });
}
```

---

### Модуль dialogs

#### `handle_action(app: &mut AssistantApp)`

**Пример из проекта (src/app/ui/dialogs.rs:105-135):**
```rust
fn handle_action(app: &mut AssistantApp) {
    match app.dialog.dialog_type {
        DialogType::PackageSearch => {
            if !app.dialog.input.is_empty() {
                app.tasks.execute(
                    BackgroundTask::SearchPackages(app.dialog.input.clone())
                );
            }
        }
        DialogType::Confirmation => {
            let title = &app.dialog.title;
            let package = &app.dialog.package;

            if title.contains("Установка") && !package.is_empty() {
                app.tasks.execute(BackgroundTask::InstallPackage(package.clone()));
            } else if title.contains("Удаление") && !package.is_empty() {
                app.tasks.execute(BackgroundTask::RemovePackage(package.clone()));
            } else if title.contains("Обновление") {
                app.tasks.execute(BackgroundTask::UpdateSystem);
            } else if package == "__shutdown__" {
                app.tasks.execute(BackgroundTask::ShutdownSystem);
            } else if package == "__reboot__" {
                app.tasks.execute(BackgroundTask::RebootSystem);
            }
        }
        DialogType::Info => {}
    }

    app.dialog.hide();
}
```

---

## Модуль desktop

**Файл:** `src/app/desktop.rs`

### DesktopEnvironment::detect()

**Пример из проекта (src/app/desktop.rs:17-58):**
```rust
pub fn detect() -> Self {
    // Проверяем XDG_CURRENT_DESKTOP
    if let Ok(desktop) = env::var("XDG_CURRENT_DESKTOP") {
        let desktop = desktop.to_lowercase();
        if desktop.contains("gnome") || desktop.contains("unity") || desktop.contains("budgie") {
            return Self::Gnome;
        }
        if desktop.contains("kde") || desktop.contains("plasma") {
            return Self::Kde;
        }
        if desktop.contains("xfce") {
            return Self::Xfce;
        }
    }

    // Проверяем DESKTOP_SESSION
    if let Ok(session) = env::var("DESKTOP_SESSION") {
        let session = session.to_lowercase();
        if session.contains("gnome") || session.contains("ubuntu") {
            return Self::Gnome;
        }
        if session.contains("plasma") || session.contains("kde") {
            return Self::Kde;
        }
    }

    // Проверяем KDE_FULL_SESSION
    if env::var("KDE_FULL_SESSION").is_ok() {
        return Self::Kde;
    }

    // Проверяем GNOME_DESKTOP_SESSION_ID
    if env::var("GNOME_DESKTOP_SESSION_ID").is_ok() {
        return Self::Gnome;
    }

    Self::Other
}
```

---

### DesktopEnvironment::terminal_priority()

**Пример из проекта (src/app/desktop.rs:71-91):**
```rust
pub fn terminal_priority(&self) -> Vec<&'static str> {
    match self {
        Self::Gnome => vec![
            "gnome-terminal",
            "kgx", // GNOME Console
            "alacritty",
            "kitty",
            "xterm",
        ],
        Self::Kde => vec!["konsole", "alacritty", "kitty", "xterm"],
        Self::Xfce => vec!["xfce4-terminal", "alacritty", "kitty", "xterm"],
        Self::Other => vec![
            "alacritty",
            "kitty",
            "gnome-terminal",
            "konsole",
            "xfce4-terminal",
            "xterm",
        ],
    }
}
```

---

### DeStyles::for_de()

**Пример из проекта (src/app/desktop.rs:111-132):**
```rust
pub fn for_de(de: DesktopEnvironment) -> Self {
    match de {
        DesktopEnvironment::Gnome => Self {
            rounding: 12.0, // GNOME использует более округлые формы
            spacing: 12.0,
        },
        DesktopEnvironment::Kde => Self {
            rounding: 6.0, // KDE более строгий
            spacing: 10.0,
        },
        DesktopEnvironment::Xfce => Self {
            rounding: 4.0, // Xfce минималистичный
            spacing: 8.0,
        },
        DesktopEnvironment::Other => Self {
            rounding: 8.0,
            spacing: 10.0,
        },
    }
}
```

---

## Модуль installer

**Файл:** `src/app/installer.rs`

### install() -> InstallResult

**Пример из проекта (src/app/installer.rs:37-124):**
```rust
pub fn install() -> InstallResult {
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => {
            return InstallResult {
                message: "[X] Не удалось определить домашнюю директорию".into(),
            }
        }
    };

    // Находим текущий бинарник
    let current_exe = match std::env::current_exe() {
        Ok(p) => p,
        Err(e) => {
            return InstallResult {
                message: format!("[X] Не удалось найти исполняемый файл: {}", e),
            }
        }
    };

    // Создаём директории
    let bin_dir = home.join(".local/bin");
    let desktop_dir = home.join(".local/share/applications");
    let icon_dir = home.join(".local/share/icons");

    for dir in [&bin_dir, &desktop_dir, &icon_dir] {
        if let Err(e) = fs::create_dir_all(dir) {
            return InstallResult {
                message: format!("[X] Не удалось создать директорию: {}", e),
            };
        }
    }

    // Копируем бинарник
    let bin_path = home.join(INSTALL_BIN_PATH);
    if let Err(e) = fs::copy(&current_exe, &bin_path) {
        return InstallResult {
            message: format!("[X] Не удалось скопировать бинарник: {}", e),
        };
    }

    // Устанавливаем права на исполнение
    if let Err(e) = fs::set_permissions(&bin_path, fs::Permissions::from_mode(0o755)) {
        return InstallResult {
            message: format!("[X] Не удалось установить права: {}", e),
        };
    }

    // Создаём .desktop файл
    let desktop_path = home.join(DESKTOP_FILE_PATH);
    let desktop_content = generate_desktop_file(&bin_path, &icon_path);
    if let Err(e) = fs::write(&desktop_path, desktop_content) {
        return InstallResult {
            message: format!("[X] Не удалось создать .desktop файл: {}", e),
        };
    }

    // Обновляем кэш desktop-файлов
    let _ = Command::new("update-desktop-database").arg(desktop_dir).output();

    InstallResult {
        message: format!(
            "[OK] Альфонс установлен!\n\
             Бинарник: {}\n\
             Ярлык добавлен в меню приложений.",
            bin_path.display()
        ),
    }
}
```

---

### generate_desktop_file()

**Пример из проекта (src/app/installer.rs:227-244):**
```rust
fn generate_desktop_file(bin_path: &Path, icon_path: &Path) -> String {
    format!(
        r#"[Desktop Entry]
Name=Альфонс
GenericName=AI Assistant
Comment=Помощник для Arch Linux с AI интеграцией
Exec={}
Icon={}
Terminal=false
Type=Application
Categories=Utility;System;
Keywords=arch;linux;ai;assistant;ollama;
StartupNotify=true
"#,
        bin_path.display(),
        icon_path.display()
    )
}
```

---

## Модуль constants

**Файл:** `src/app/constants.rs`

### Все константы

**Пример из проекта (src/app/constants.rs:1-65):**
```rust
// === Приложение ===
pub const APP_NAME: &str = "Альфонс";
pub const APP_VERSION: &str = "0.0.5";
pub const DEFAULT_ASSISTANT_NAME: &str = "Альфонс";
pub const DEFAULT_ACCENT_COLOR: [u8; 3] = [61, 174, 233]; // Голубой

// === Ollama AI ===
pub const OLLAMA_URL: &str = "http://localhost:11434/api/generate";
pub const OLLAMA_MODEL: &str = "llama3";
pub const OLLAMA_CUSTOM_MODEL: &str = "alfons";
pub const OLLAMA_TIMEOUT_SECS: u64 = 60;
pub const OLLAMA_INSTALL_SCRIPT: &str = "https://ollama.com/install.sh";

// === Yay (AUR) ===
pub const YAY_INSTALL_DIR: &str = "/tmp/yay-install";
pub const YAY_AUR_URL: &str = "https://aur.archlinux.org/yay.git";

// === Пути ===
pub const CONFIG_APP_NAME: &str = "alfons-assistant";

// === Лимиты ===
pub const MAX_CHAT_MESSAGES: usize = 100;

// === UI ===
pub const SETTINGS_PANEL_WIDTH: f32 = 280.0;

// === Сообщения ===
pub mod messages {
    pub const WELCOME: &str = "Система готова. Введите команду или задайте вопрос ИИ.";
    pub const CHAT_CLEARED: &str = "История чата очищена. Чем могу помочь?";
    pub const PROCESSING: &str = "Обработка...";
    pub const MODEL_CREATING: &str =
        "Создаю кастомную модель 'alfons'... Это может занять несколько минут.";
    pub const MODEL_CREATED: &str = "[OK] Модель 'alfons' создана! Переключаю на неё.";
    pub const MODEL_EXISTS: &str = "Модель 'alfons' уже существует.";
    pub const OLLAMA_INSTALLING: &str = "Устанавливаю Ollama...";
    pub const OLLAMA_INSTALLED: &str = "[OK] Ollama успешно установлена!";
    pub const YAY_INSTALLING: &str = "Устанавливаю yay...";
    pub const YAY_INSTALLED: &str = "[OK] yay успешно установлен!";
}

// === Ошибки ===
pub mod errors {
    pub const OLLAMA_CONNECTION: &str = "Ошибка связи с Ollama. Убедитесь, что сервис запущен.";
    pub const OLLAMA_PARSE: &str = "Ошибка обработки ответа от Ollama.";
    pub const PACKAGE_NOT_FOUND: &str = "Ничего не найдено.";
    pub const MODEL_CREATE_FAILED: &str =
        "[X] Не удалось создать модель. Проверьте Ollama и llama3.";
    pub const MODEL_BASE_NOT_FOUND: &str =
        "[X] Базовая модель llama3 не найдена. Выполните: ollama pull llama3";
    pub const YAY_DEPS_FAILED: &str = "[X] Не удалось установить зависимости для yay.";
    pub const YAY_CLONE_FAILED: &str = "[X] Не удалось склонировать репозиторий yay.";
    pub const YAY_BUILD_FAILED: &str = "[X] Не удалось собрать yay.";
}
```

---

## Сводная таблица функций

| Модуль | Функция | Описание |
|--------|---------|----------|
| **assistant_app** | `new()` | Создание приложения |
| | `process_input()` | Обработка ввода |
| | `send_to_ai()` | Отправка в AI |
| | `check_tasks()` | Проверка фоновых задач |
| | `process_ai_commands()` | Обработка `[CMD:...]` |
| | `clear_chat()` | Очистка чата |
| **chat** | `DialogState::show_confirm()` | Диалог подтверждения |
| | `ChatHistory::add_message()` | Добавить сообщение |
| | `TaskManager::execute()` | Запуск фоновой задачи |
| | `InputHistory::up/down()` | Навигация по истории |
| **config** | `Config::load()` | Загрузка настроек |
| | `Config::save()` | Сохранение настроек |
| **ai** | `LocalAi::generate()` | Генерация ответа AI |
| | `check_ollama_status()` | Проверка Ollama |
| | `create_custom_model()` | Создание модели |
| | `install_ollama()` | Установка Ollama |
| | `ToolRegistry::execute()` | Выполнение инструмента |
| **commands** | `process_command()` | Главный обработчик |
| | `process_basic_command()` | Базовые команды |
| | `process_package_command()` | Пакетный менеджер |
| | `process_system_command()` | Системные команды |
| | `process_guide_command()` | Гайды |
| | `search_packages()` | Поиск пакетов |
| | `install_package()` | Установка пакета |
| | `install_yay()` | Установка yay |
| **guides** | `GuideRegistry::get()` | Получить гайд |
| | `GuideRegistry::search()` | Поиск гайдов |
| | `Guide::format()` | Форматирование |
| **ui** | `render()` | Главный рендеринг |
| | `render_message()` | Пузырь сообщения |
| | `dialogs::render()` | Модальный диалог |
| **desktop** | `DesktopEnvironment::detect()` | Определение DE |
| | `terminal_priority()` | Приоритет терминалов |
| | `DeStyles::for_de()` | Стили для DE |
| **installer** | `install()` | Установка в систему |
| | `uninstall()` | Удаление из системы |
| | `is_installed()` | Проверка установки |
