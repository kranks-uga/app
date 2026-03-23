# Документация функций Gаврик

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
- [Модуль command_log](#модуль-command_log)
- [Модуль constants](#модуль-constants)

---

## Модуль assistant_app

**Файл:** `src/app/assistant_app.rs`

Главная структура приложения, объединяющая все компоненты.

### Константы

```rust
/// Интервал проверки статуса Ollama (в секундах)
const OLLAMA_CHECK_INTERVAL: u64 = 30;
```

### Вспомогательные функции

#### `cmd_regex() -> &'static Regex`

Возвращает статически инициализированный Regex для парсинга `[CMD:...]` маркеров в ответах AI.

```rust
fn cmd_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\[CMD:([^\]]+)\]").expect("Invalid CMD regex"))
}
```

### Структура `AssistantApp`

```rust
pub struct AssistantApp {
    // Данные
    pub config: Config,
    pub chat: ChatHistory,
    pub guides: GuideRegistry,
    pub ai: Arc<LocalAi>,

    // UI состояние
    pub input_text: String,
    pub show_settings: bool,
    pub dialog: DialogState,
    pub input_history: InputHistory,
    pub ollama_online: Arc<AtomicBool>,
    pub ollama_installed: Arc<AtomicBool>,
    pub yay_installed: Arc<AtomicBool>,
    pub custom_model_exists: Arc<AtomicBool>,
    pub app_installed: Arc<AtomicBool>,
    last_ollama_check: Instant,

    // Окружение рабочего стола
    pub desktop_env: DesktopEnvironment,
    pub de_styles: DeStyles,

    // Фоновые задачи
    pub tasks: TaskManager,
    task_receiver: mpsc::Receiver<String>,
}
```

### Функции

#### `AssistantApp::new(cc: &CreationContext) -> Self`

Создаёт новый экземпляр приложения. Выполняет:
1. Создание `TaskManager` и канала для результатов
2. Загрузку конфигурации
3. Определение окружения рабочего стола (DE)
4. Инициализацию чата с приветственным сообщением
5. Создание AI клиента и установку модели
6. Фоновые проверки: Ollama online, Ollama установлена, yay установлен, кастомная модель, установка в систему

```rust
let app = AssistantApp::new(cc);
```

---

#### `check_ollama_periodic(&mut self)`

Периодически проверяет статус Ollama (каждые 30 секунд). Вызывается на каждом кадре UI из `update()`. Запускает асинхронную проверку через `tokio::spawn`.

```rust
fn check_ollama_periodic(&mut self) {
    if self.last_ollama_check.elapsed() >= Duration::from_secs(OLLAMA_CHECK_INTERVAL) {
        self.last_ollama_check = Instant::now();
        let ollama_online = self.ollama_online.clone();
        tokio::spawn(async move {
            let status = check_ollama_status().await;
            ollama_online.store(status, Ordering::SeqCst);
        });
    }
}
```

---

#### `process_input(&mut self)`

Обрабатывает ввод пользователя. Сначала пробует распознать как команду через `commands::process_command()`, если не удалось — отправляет в AI.

```rust
app.process_input();
// 1. Сохраняет ввод в историю
// 2. Добавляет сообщение пользователя в чат
// 3. Пробует обработать как команду
// 4. Если не команда — отправляет в AI
// 5. Очищает поле ввода
```

---

#### `send_to_ai(&self, input: &str)`

Отправляет запрос в AI асинхронно. Передаёт **историю чата** для контекста.

```rust
fn send_to_ai(&self, input: &str) {
    let history = self.chat.as_pairs(); // История для контекста
    tokio::spawn(async move {
        let response = ai.generate(&history, &input).await;
        let _ = tx.send(response);
    });
}
```

---

#### `check_tasks(&mut self)`

Проверяет завершённые фоновые задачи. AI ответы (начинающиеся с имени ассистента) обрабатываются через `process_ai_commands()`, остальные отображаются как системные сообщения.

```rust
app.check_tasks();
// Вызывается на каждом кадре UI из update()
```

---

#### `process_ai_commands(&mut self, text: &str) -> String`

Обрабатывает маркеры `[CMD:...]` в ответе AI и выполняет соответствующие команды.

**Пример:**
```
Вход: "Сейчас установлю [CMD:установить firefox]"
Выход: "Сейчас установлю " (команда выполнится — откроется диалог подтверждения)
```

---

#### `clear_chat(&mut self)`

Очищает историю чата и добавляет сообщение «История чата очищена».

```rust
app.clear_chat();
```

---

#### `impl eframe::App for AssistantApp`

Реализация трейта `App` для интеграции с eframe. Метод `update()` вызывается на каждом кадре:
1. `check_tasks()` — проверка фоновых задач
2. `check_ollama_periodic()` — проверка статуса Ollama
3. Применение стилей DE (скругления, отступы)
4. `ui::render()` — отрисовка интерфейса

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

Состояние диалогового окна.

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

#### `DialogState::new() -> Self`

Создаёт пустой диалог (скрытый).

#### `show_search(&mut self)`

Открывает диалог поиска пакетов.

```rust
dialog.show_search();
// title: "Поиск пакетов"
// message: "Введите название пакета:"
```

#### `show_confirm(&mut self, title: &str, message: &str, package: &str)`

Открывает диалог подтверждения действия.

```rust
dialog.show_confirm(
    "Установка пакета",
    "Установить 'firefox' через yay?",
    "firefox",
);
```

**Специальные значения `package`:**
- `"__shutdown__"` — выключение компьютера
- `"__reboot__"` — перезагрузка

#### `hide(&mut self)`

Скрывает диалог и очищает поля ввода.

---

### Перечисление `BackgroundTask`

Типы фоновых задач, выполняемых в отдельном потоке.

```rust
pub enum BackgroundTask {
    SearchPackages(String),    // Поиск пакетов через yay
    InstallPackage(String),    // Установка пакета
    RemovePackage(String),     // Удаление пакета
    UpdateSystem,              // Обновление системы (yay -Syu)
    InstallYay,                // Установка yay из AUR
    ShutdownSystem,            // Выключение ПК
    RebootSystem,              // Перезагрузка ПК
    CreateCustomModel,         // Создание модели alfons
    InstallToSystem,           // Установка приложения в систему
    UninstallFromSystem,       // Удаление приложения из системы
    InstallOllama,             // Установка Ollama
    StartOllama,               // Запуск сервиса Ollama
}
```

---

### Структура `ChatMessage`

```rust
pub struct ChatMessage {
    pub sender: String,              // "Вы", "Gаврик", "Система"
    pub text: String,                // Текст сообщения
    pub timestamp: DateTime<Local>,  // Время отправки
}
```

---

### Структура `ChatHistory`

Управление историей чата с ограничением количества сообщений (VecDeque).

#### `ChatHistory::new(max_messages: usize) -> Self`

Создаёт историю с указанным лимитом. По умолчанию — `MAX_CHAT_MESSAGES` (100).

#### `add_message(&mut self, sender: impl Into<String>, text: impl Into<String>)`

Добавляет сообщение. При превышении лимита удаляет самое старое (O(1)).

```rust
app.chat.add_message("Система", "Ollama запущена!");
app.chat.add_message(&config.assistant_name, "Привет!");
app.chat.add_message("Вы", &input);
```

#### `clear(&mut self)`

Очищает всю историю.

#### `messages(&self) -> impl Iterator<Item = &ChatMessage>`

Возвращает итератор по сообщениям для отрисовки в UI.

#### `as_pairs(&self) -> Vec<(String, String)>`

Возвращает историю как вектор пар `(отправитель, текст)`. Используется для передачи контекста в AI.

```rust
let history = app.chat.as_pairs();
ai.generate(&history, &input).await;
```

---

### Структура `TaskManager`

Менеджер фоновых задач с каналами mpsc.

#### `TaskManager::new() -> (Self, Receiver<String>)`

Создаёт менеджер и запускает фоновый поток-обработчик. Возвращает кортеж `(менеджер, канал_результатов)`.

```rust
let (tasks, task_receiver) = TaskManager::new();
```

#### `execute(&self, task: BackgroundTask)`

Запускает фоновую задачу. Устанавливает флаг `is_processing` перед отправкой.

```rust
app.tasks.execute(BackgroundTask::SearchPackages("firefox".into()));
app.tasks.execute(BackgroundTask::InstallPackage("vim".into()));
app.tasks.execute(BackgroundTask::UpdateSystem);
app.tasks.execute(BackgroundTask::StartOllama);
```

#### `is_busy(&self) -> bool`

Проверяет, выполняется ли задача. Используется для отображения индикатора загрузки.

---

### Структура `InputHistory`

История введённых команд для навигации стрелками (до 50 записей).

#### `InputHistory::new() -> Self`

Создаёт пустую историю.

#### `push(&mut self, input: &str)`

Добавляет команду. Не добавляет дубликаты подряд и пустые строки.

#### `up(&mut self, current: &str) -> Option<&str>`

Переход вверх (предыдущая команда). При первом вызове сохраняет текущий ввод.

#### `down(&mut self) -> Option<&str>`

Переход вниз (следующая команда). При достижении конца возвращает сохранённый ввод.

#### `reset(&mut self)`

Сбрасывает позицию навигации. Вызывается при ручном вводе текста.

---

## Модуль config

**Файл:** `src/app/config.rs`

### Структура `Config`

```rust
#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub assistant_name: String,        // Имя ассистента (по умолчанию "Gаврик")
    pub accent_color: [u8; 3],         // RGB цвет акцента (по умолчанию [61, 174, 233])
    pub ollama_model: String,          // Модель Ollama (по умолчанию "llama3")
}
```

**Путь файла:** `~/.config/alfons-assistant/config.toml`

**Пример JSON:**
```json
{
  "assistant_name": "Gаврик",
  "accent_color": [61, 174, 233],
  "ollama_model": "alfons"
}
```

#### `Config::load() -> Self`

Загружает конфигурацию через `confy`. Возвращает значения по умолчанию при ошибке.

```rust
let config = Config::load();
```

#### `Config::save(&self) -> Result<(), String>`

Сохраняет конфигурацию на диск.

```rust
if let Err(e) = app.config.save() {
    app.chat.add_message("Система", &e);
}
```

#### `accent_color_egui(&self) -> egui::Color32`

Конвертирует `[u8; 3]` в `egui::Color32` для использования в UI.

```rust
let accent = app.config.accent_color_egui();
```

---

## Модуль ai

**Файл:** `src/app/ai/`

### Модуль local_provider

**Файл:** `src/app/ai/local_provider.rs`

AI клиент, работающий через **Ollama Chat API** (`/api/chat`) с поддержкой истории диалога.

#### Внутренние структуры

```rust
/// Сообщение для Chat API
struct ChatMessage {
    role: String,    // "system", "user", "assistant"
    content: String,
}

struct OllamaChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    stream: bool,
}

struct OllamaChatResponse {
    message: ChatMessageContent,
}

struct ChatMessageContent {
    content: String,
}
```

### Структура `LocalAi`

```rust
pub struct LocalAi {
    client: Client,           // HTTP клиент (reqwest) с таймаутом 60с
    model: RwLock<String>,    // Текущая модель (потокобезопасная)
    tools: ToolRegistry,      // Реестр инструментов
}
```

#### `LocalAi::new() -> Self`

Создаёт клиент с таймаутом `OLLAMA_TIMEOUT_SECS` (60 секунд) и моделью по умолчанию.

#### `set_model(&self, model: &str)`

Устанавливает модель. Потокобезопасно через `RwLock`.

```rust
ai.set_model("alfons");
ai.set_model("llama3");
```

#### `get_model(&self) -> String`

Возвращает текущую модель.

#### `ensure_model(&self) -> Result<(), String>`

*(Приватный, async)* Проверяет наличие модели и автоматически устанавливает при отсутствии:
1. Проверяет `ollama show <model>`
2. Если модель `alfons` — сначала качает базовую `llama3`, затем создаёт кастомную
3. Для других моделей — `ollama pull <model>`

Вызывается автоматически перед каждым запросом к AI.

#### `pull_model(model: &str) -> Result<(), String>`

*(Приватный, async)* Скачивает модель через `ollama pull`.

#### `create_custom_model_auto(&self) -> Result<(), String>`

*(Приватный, async)* Создаёт кастомную модель автоматически. Находит или генерирует Modelfile, затем вызывает `ollama create`.

#### `find_or_create_modelfile() -> Result<PathBuf, String>`

*(Приватный)* Ищет Modelfile в стандартных местах или создаёт новый в конфиг-директории.

**Порядок поиска:**
1. Рядом с исполняемым файлом
2. Текущая директория
3. `~/.config/alfons-assistant/Modelfile`

#### `generate(&self, history: &[(String, String)], input: &str) -> Result<String, String>`

Генерирует ответ AI с учётом истории чата. Основной метод взаимодействия с Ollama.

**Этапы:**
1. `ensure_model()` — проверка/установка модели
2. Формирование сообщений: system prompt + история + текущий ввод
3. POST запрос к `/api/chat`
4. `process_response()` — обработка `[TOOL:...]` маркеров

```rust
let history = chat.as_pairs();
let response = ai.generate(&history, "Который час?").await?;
// Ответ: "Сейчас 14:30:25"
```

**Как формируется история:**
- `"Вы"` → role: `"user"`
- `"Система"` → пропускается
- Всё остальное → role: `"assistant"`

#### `process_response(&self, response: &str) -> String`

*(Приватный)* Обрабатывает маркеры `[TOOL:...]` в ответе AI, заменяя их результатами инструментов. Маркеры `[CMD:...]` оставляет как есть для обработки в `assistant_app`.

```
Вход: "Сейчас [TOOL:время], дата: [TOOL:дата]"
Выход: "Сейчас 14:30:25, дата: 07.02.2026"
```

---

#### `check_ollama_status() -> bool`

*(async)* Проверяет, запущен ли сервис Ollama. Отправляет GET на `http://localhost:11434/api/tags` с таймаутом 2 секунды.

```rust
let online = check_ollama_status().await; // true/false
```

#### `is_custom_model_exists() -> bool`

Проверяет, существует ли кастомная модель `alfons` через `ollama show alfons`.

#### `is_base_model_exists() -> bool`

Проверяет, существует ли базовая модель `llama3` через `ollama show llama3`.

#### `create_custom_model() -> String`

Создаёт кастомную модель `alfons` из Modelfile. Если базовая модель не найдена — показывает диалог `rfd` с предложением скачать.

**Порядок действий:**
1. Проверяет наличие базовой модели → предлагает скачать при отсутствии
2. Проверяет, не существует ли уже `alfons`
3. Ищет Modelfile или создаёт новый
4. Выполняет `ollama create alfons -f <Modelfile>`

#### `generate_modelfile_content() -> &'static str`

*(Приватный)* Возвращает содержимое Modelfile с системным промптом, инструментами и правилами поведения AI.

#### `is_ollama_installed() -> bool`

Проверяет, установлена ли Ollama через `which ollama`.

#### `install_ollama() -> String`

Устанавливает Ollama через официальный скрипт (`curl | sh`) в терминале. Возвращает сообщение о результате.

```rust
let msg = install_ollama(); // "[OK] Установка Ollama запущено в kitty"
```

#### `start_ollama_service() -> String`

Запускает `ollama serve` в фоне. Ждёт 2 секунды перед возвратом.

```rust
let msg = start_ollama_service(); // "[OK] Сервис Ollama запущен!"
```

---

### Модуль tools

**Файл:** `src/app/ai/tools.rs`

### Структура `Tool`

```rust
pub struct Tool {
    pub name: String,          // Имя инструмента ("время", "дата" и т.д.)
    pub description: String,   // Описание для системного промпта
    pub handler: ToolHandler,  // Функция-обработчик fn() -> String
}
```

### Структура `ToolRegistry`

Контейнер для всех инструментов AI.

#### `ToolRegistry::new() -> Self`

Создаёт реестр со следующими встроенными инструментами:

| Инструмент | Описание | Пример вывода |
|------------|----------|---------------|
| `время` | Текущее время | `14:30:25` |
| `дата` | Текущая дата | `07.02.2026` |
| `дата_и_время` | Дата и время | `07.02.2026 14:30:25` |
| `список_гайдов` | Список доступных гайдов | `pacman, aur, wifi, systemd, gpu, audio, locale, backup` |
| `память` | Использование RAM | `4.2G / 16G (использовано)` |
| `диск` | Использование дисков | `45G / 100G (48%)` |
| `cpu` | Информация о CPU | `AMD Ryzen 5 5600X (загрузка: 1.23)` |
| `система` | Общая информация | Память + CPU + Диск |

#### `register(&mut self, name: &str, description: &str, handler: ToolHandler)`

Регистрирует новый инструмент.

```rust
registry.register(
    "время",
    "получить текущее время",
    || Local::now().format("%H:%M:%S").to_string(),
);
```

#### `execute(&self, name: &str) -> Option<String>`

Выполняет инструмент по имени. Возвращает `None` если инструмент не найден.

```rust
let time = registry.execute("время"); // Some("14:30:25")
let none = registry.execute("неизвестный"); // None
```

#### `generate_system_prompt(&self) -> String`

Генерирует системный промпт с описанием всех инструментов и команд для AI. Включает:
- Список `[TOOL:...]` инструментов
- Список `[CMD:...]` команд
- Правила поведения AI
- Примеры использования

---

#### Вспомогательные функции

#### `get_memory_info() -> String`

Получает использование RAM через `free -h --si`. Парсит вторую строку вывода.

**Пример:** `"4.2G / 16G (использовано)"`

#### `get_disk_info() -> String`

Получает использование корневого раздела через `df -h /`.

**Пример:** `"45G / 100G (48%)"`

#### `get_cpu_info() -> String`

Получает имя CPU из `/proc/cpuinfo` и загрузку из `/proc/loadavg`.

**Пример:** `"AMD Ryzen 5 5600X 6-Core Processor (загрузка: 1.23)"`

---

## Модуль commands

**Файл:** `src/app/commands/`

### Главная функция process_command

Обрабатывает команду и возвращает ответ. Проверяет в порядке приоритета:
1. Базовые команды (время, дата, помощь)
2. Системные команды (выключение, перезагрузка)
3. Пакетный менеджер
4. Гайды

Логирует все распознанные команды через `command_log`.

```rust
let response = commands::process_command(
    &input,           // Текст команды
    &assistant_name,  // Имя ассистента для приветствия
    &mut dialog,      // Состояние диалога
    &tasks,           // Менеджер задач
    &guides,          // Реестр гайдов
);
// Some("Текущее время: 14:30:25") или None если не команда
```

---

### Модуль base

**Файл:** `src/app/commands/base.rs`

#### Константа `CMD_CLEAR_CHAT`

```rust
pub const CMD_CLEAR_CHAT: &str = "COMMAND_ACTION_CLEAR";
```

Маркер, перехватываемый в `assistant_app` для очистки чата.

#### `process_basic_command(cmd: &str, assistant_name: &str) -> Option<String>`

| Команда | Алиасы | Действие |
|---------|--------|----------|
| `привет` | `здравствуй`, `хай`, `hello` | Приветствие |
| `очистить` | `очистить чат`, `clear` | Возвращает `CMD_CLEAR_CHAT` |
| `скажи <текст>` | — | Повторяет текст |
| `время` | `который час`, `time` | Текущее время |
| `дата` | `какое сегодня число`, `date` | Текущая дата |
| `дата и время` | — | Дата и время |
| `помощь` | `help`, `?` | Справка по командам |

---

### Модуль package

**Файл:** `src/app/commands/package.rs`

#### `process_package_command(cmd: &str, dialog: &mut DialogState, tasks: &TaskManager) -> Option<String>`

| Команда | Действие |
|---------|----------|
| `поиск пакетов`, `найти пакеты` | Открывает диалог поиска |
| `установить <пакет>` | Открывает диалог подтверждения установки |
| `удалить <пакет>` | Открывает диалог подтверждения удаления |
| `обновить систему`, `обновить`, `обновление` | Диалог подтверждения обновления |
| `поиск <запрос>` | Запускает фоновый поиск через yay |

#### `search_packages(query: &str) -> String`

Поиск пакетов через `yay -Ss <query>`. Возвращает результат или "Ничего не найдено."

#### `install_package(package: &str) -> String`

Запускает `yay -S <пакет>` в терминале для интерактивного sudo.

#### `remove_package(package: &str) -> String`

Запускает `yay -R <пакет>` в терминале.

#### `update_system() -> String`

Запускает `yay -Syu` в терминале.

#### `is_yay_installed() -> bool`

Проверяет наличие yay через `which yay`.

#### `install_yay() -> String`

Устанавливает yay из AUR:
1. Устанавливает зависимости (`git`, `base-devel`) через `pkexec pacman`
2. Клонирует `https://aur.archlinux.org/yay.git` в `/tmp/yay-install`
3. Собирает через `makepkg -si --noconfirm`
4. Очищает `/tmp/yay-install`

#### `run_in_terminal(cmd: &str, action: &str) -> String`

*(Приватный)* Запускает команду в терминале с учётом текущего DE.

---

### Модуль system

**Файл:** `src/app/commands/system.rs`

#### `process_system_command(cmd: &str, dialog: &mut DialogState) -> Option<String>`

| Команда | Алиасы | Действие |
|---------|--------|----------|
| `выключить пк` | `выключить компьютер` | Диалог подтверждения выключения |
| `перезагрузить` | `рестарт` | Диалог подтверждения перезагрузки |

#### `execute_shutdown() -> String`

Выполняет `shutdown -h now`. Вызывается после подтверждения.

#### `execute_reboot() -> String`

Выполняет `shutdown -r now`. Вызывается после подтверждения.

---

### Модуль guide

**Файл:** `src/app/commands/guide.rs`

#### `process_guide_command(cmd: &str, guides: &GuideRegistry) -> Option<String>`

| Команда | Действие |
|---------|----------|
| `гайды`, `guides`, `обучение` | Список всех гайдов |
| `гайд <тема>`, `guide <тема>` | Показать гайд (точное совпадение или поиск) |
| `найти гайд <запрос>`, `поиск гайдов <запрос>` | Поиск по гайдам |

Если по запросу найден один гайд — показывает его. Если несколько — показывает список.

---

### Модуль applications (заглушка)

**Файл:** `src/app/commands/applications.rs`

Заготовка для будущей поддержки установки популярных приложений. Пока не подключён к `process_command()`.

---

## Модуль guides

**Файл:** `src/app/guides/mod.rs`

### Структура `GuideStep`

```rust
pub struct GuideStep {
    pub instruction: String,       // Текст инструкции
    pub command: Option<String>,   // Команда терминала (опционально)
    pub note: Option<String>,      // Примечание (опционально)
}
```

**Builder-паттерн:**
```rust
GuideStep::new("Установить пакет")
    .with_command("sudo pacman -S <пакет>")
    .with_note("Потребуется пароль root")
```

---

### Структура `Guide`

```rust
pub struct Guide {
    pub id: String,            // Идентификатор ("pacman", "wifi")
    pub title: String,         // Заголовок
    pub description: String,   // Описание
    pub steps: Vec<GuideStep>, // Шаги
    pub tags: Vec<String>,     // Теги для поиска
}
```

**Builder-паттерн:**
```rust
Guide::new("pacman", "Основы Pacman", "Базовые команды...")
    .add_tags(&["пакеты", "установка"])
    .add_step(GuideStep::new("Обновить").with_command("sudo pacman -Syu"))
```

#### `Guide::format(&self) -> String`

Форматирует гайд для вывода в чат.

**Пример вывода:**
```
 Основы Pacman
Базовые команды пакетного менеджера Arch Linux

1. Обновить список пакетов и систему
   $ sudo pacman -Syu
   ℹ Рекомендуется делать перед установкой новых пакетов
```

---

### Структура `GuideRegistry`

Контейнер всех гайдов (HashMap по id).

#### `GuideRegistry::new() -> Self`

Создаёт реестр и регистрирует встроенные гайды.

#### `register(&mut self, guide: Guide)`

Регистрирует гайд.

#### `get(&self, id: &str) -> Option<&Guide>`

Получить гайд по точному ID.

#### `search(&self, query: &str) -> Vec<&Guide>`

Поиск по ключевому слову в названии, описании, тегах и ID (без учёта регистра).

#### `format_list(&self) -> String`

Форматирует список всех гайдов, отсортированный по названию.

#### Встроенные гайды

| ID | Название | Теги |
|----|----------|------|
| `pacman` | Основы Pacman | пакеты, установка, обновление |
| `aur` | Работа с AUR | yay, aur, репозиторий |
| `wifi` | Настройка WiFi | сеть, интернет, wireless |
| `systemd` | Управление сервисами | сервисы, службы, демоны |
| `gpu` | Драйверы видеокарты | nvidia, amd, intel |
| `audio` | Настройка звука | pipewire, pulseaudio |
| `locale` | Локализация системы | язык, раскладка, клавиатура |
| `backup` | Резервное копирование | timeshift, снимки |

---

## Модуль ui

**Файл:** `src/app/ui/`

### Главный модуль (mod.rs)

#### `render(ctx: &egui::Context, app: &mut AssistantApp)`

Главная функция рендеринга. Вызывает:
1. `handle_hotkeys()` — горячие клавиши
2. `render_header()` — шапка
3. `render_settings()` — панель настроек (если открыта)
4. `render_input()` — поле ввода
5. `render_chat()` — область чата
6. `dialogs::render()` — модальный диалог (если виден)

#### `handle_hotkeys(ctx, app)`

| Клавиша | Действие |
|---------|----------|
| `Ctrl+L` | Очистить чат |
| `Escape` | Закрыть диалог или панель настроек |

#### `render_header(ctx, app, accent)`

Отображает шапку с:
- Названием ассистента (КАПСОМ, цвет акцента)
- Индикатором Ollama (`[ON]`/`[OFF]`)
- Кнопкой настроек `[=]`
- Индикатором загрузки «Обработка...»

#### `render_settings(ctx, app, accent)`

Боковая панель настроек (ширина 280px) с прокруткой:
- **Персонализация:** выбор цвета темы
- **ИИ (Ollama):** статус, установка, запуск, выбор модели, создание кастомной модели
- **Чат:** кнопка очистки
- **Пакетный менеджер:** статус yay, установка
- **Горячие клавиши:** справка
- **О программе:** версия, DE
- **Установка:** установка/удаление из системы, проверка PATH

#### `render_chat(ctx, app, accent)`

Область чата с автопрокруткой к последнему сообщению.

#### `render_input(ctx, app, accent)`

Нижняя панель с полем ввода и кнопкой «ОТПРАВИТЬ». Поддержка:
- Enter для отправки
- Стрелки вверх/вниз для навигации по истории

---

### Модуль widgets

**Файл:** `src/app/ui/widgets.rs`

#### `render_message(ui, msg, accent)`

Отрисовывает сообщение в виде «пузыря» (bubble):
- Сообщения пользователя — справа, синий фон
- Сообщения ассистента/системы — слева, серый фон с акцентом
- Заголовок: имя + время
- Скругления: разные для пользователя и ассистента
- Максимальная ширина 70%
- Клик по тексту — копирование в буфер обмена

---

### Модуль dialogs

**Файл:** `src/app/ui/dialogs.rs`

#### `render(ctx, app, accent)`

Модальный диалог с затемнением фона (чёрный с 63% прозрачности). Содержит:
- Заголовок
- Сообщение
- Поле ввода (для `PackageSearch`)
- Имя пакета (для `Confirmation`)
- Кнопки «Отмена» и действие

#### `handle_action(app: &mut AssistantApp)`

*(Приватный)* Обрабатывает подтверждение:
- `PackageSearch` → `BackgroundTask::SearchPackages`
- `Confirmation` → определяет действие по заголовку:
  - "Установка" → `BackgroundTask::InstallPackage`
  - "Удаление" → `BackgroundTask::RemovePackage`
  - "Обновление" → `BackgroundTask::UpdateSystem`
  - `__shutdown__` → `BackgroundTask::ShutdownSystem`
  - `__reboot__` → `BackgroundTask::RebootSystem`

---

## Модуль desktop

**Файл:** `src/app/desktop.rs`

### Перечисление `DesktopEnvironment`

```rust
pub enum DesktopEnvironment {
    Gnome,
    Kde,
    Xfce,
    Other,  // по умолчанию
}
```

#### `DesktopEnvironment::detect() -> Self`

Определяет DE по переменным окружения:
1. `XDG_CURRENT_DESKTOP` — gnome/unity/budgie, kde/plasma, xfce
2. `DESKTOP_SESSION` — gnome/ubuntu, plasma/kde, xfce
3. `KDE_FULL_SESSION` → Kde
4. `GNOME_DESKTOP_SESSION_ID` → Gnome
5. Иначе → Other

#### `preferred_terminal(&self) -> &'static str`

| DE | Терминал |
|----|----------|
| Gnome | `gnome-terminal` |
| Kde | `konsole` |
| Xfce | `xfce4-terminal` |
| Other | `xterm` |

#### `terminal_priority(&self) -> Vec<&'static str>`

Список терминалов в порядке приоритета для каждого DE. Используется при запуске команд.

| DE | Приоритет |
|----|-----------|
| Gnome | gnome-terminal, kgx, alacritty, kitty, xterm |
| Kde | konsole, alacritty, kitty, xterm |
| Xfce | xfce4-terminal, alacritty, kitty, xterm |
| Other | alacritty, kitty, gnome-terminal, konsole, xfce4-terminal, xterm |

#### `name(&self) -> &'static str`

Название DE для отображения: "GNOME", "KDE Plasma", "Xfce", "Linux".

---

### Структура `DeStyles`

```rust
pub struct DeStyles {
    pub rounding: f32,  // Скругление элементов UI
    pub spacing: f32,   // Отступы между элементами
}
```

#### `DeStyles::for_de(de: DesktopEnvironment) -> Self`

| DE | Скругление | Отступы |
|----|-----------|---------|
| Gnome | 12.0 | 12.0 |
| Kde | 6.0 | 10.0 |
| Xfce | 4.0 | 8.0 |
| Other | 8.0 | 10.0 |

---

#### `run_in_terminal(cmd: &str, action: &str) -> String`

Запускает команду в первом доступном терминале с учётом DE. Перебирает терминалы по приоритету, проверяя их наличие через `which`.

```rust
let result = run_in_terminal("curl -fsSL https://ollama.com/install.sh | sh", "Установка Ollama");
// "[OK] Установка Ollama запущено в kitty"
```

#### `get_terminal_args(term: &str, cmd: &str) -> Option<Vec<String>>`

*(Приватный)* Формирует аргументы запуска для конкретного терминала. Поддерживает: kitty, alacritty, gnome-terminal, kgx, konsole, xfce4-terminal, xterm.

---

## Модуль installer

**Файл:** `src/app/installer.rs`

### Константы

```rust
const INSTALL_BIN_PATH: &str = ".local/bin/alfons";
const DESKTOP_FILE_PATH: &str = ".local/share/applications/alfons.desktop";
const ICON_PATH: &str = ".local/share/icons/alfons.png";
```

### Структура `InstallResult`

```rust
pub struct InstallResult {
    pub message: String,
}
```

#### `is_installed() -> bool`

Проверяет наличие бинарника и .desktop файла.

#### `get_installed_path() -> Option<PathBuf>`

Возвращает путь к установленному бинарнику (`~/.local/bin/alfons`).

#### `install() -> InstallResult`

Устанавливает приложение в систему:
1. Находит текущий бинарник
2. Создаёт директории (`~/.local/bin`, `~/.local/share/applications`, `~/.local/share/icons`)
3. Копирует бинарник с правами 0o755
4. Ищет или генерирует иконку (PNG/SVG)
5. Создаёт `.desktop` файл
6. Обновляет кэш desktop-файлов

#### `uninstall() -> InstallResult`

Удаляет бинарник, .desktop файл и иконку.

#### `find_custom_icon() -> Option<PathBuf>`

*(Приватный)* Ищет иконку в стандартных местах:
1. Рядом с исполняемым файлом
2. Текущая директория
3. `assets/`
4. `~/.config/alfons-assistant/`

Поддерживаемые имена: `icon.png`, `icon.svg`, `alfons.png`, `alfons.svg`, `alfons-icon.png`, `alfons-icon.svg`

#### `generate_desktop_file(bin_path, icon_path) -> String`

*(Приватный)* Генерирует содержимое `.desktop` файла в формате XDG.

#### `generate_icon_svg() -> &'static str`

*(Приватный)* Генерирует SVG иконку с буквой «A» на голубом градиентном фоне.

#### `is_local_bin_in_path() -> bool`

Проверяет, есть ли `~/.local/bin` в переменной `PATH`.

#### `get_path_export_command() -> String`

Возвращает строку для добавления в `.bashrc`/`.zshrc`:
```bash
export PATH="$HOME/.local/bin:$PATH"
```

---

## Модуль command_log

**Файл:** `src/app/command_log.rs`

#### `log_command(command: &str, result: &str)`

Записывает выполненную команду в лог-файл.

**Путь:** `~/.local/share/alfons-assistant/commands.log`

**Формат записи:**
```
[2026-02-07 14:30:25] CMD: время -> Текущее время: 14:30:25
```

---

## Модуль constants

**Файл:** `src/app/constants.rs`

### Константы приложения

| Константа | Значение | Описание |
|-----------|----------|----------|
| `APP_NAME` | `"Gаврик"` | Название приложения |
| `APP_VERSION` | `"0.0.5"` | Версия |
| `DEFAULT_ASSISTANT_NAME` | `"Gаврик"` | Имя по умолчанию |
| `DEFAULT_ACCENT_COLOR` | `[61, 174, 233]` | Голубой цвет акцента |

### Константы Ollama

| Константа | Значение | Описание |
|-----------|----------|----------|
| `OLLAMA_CHAT_URL` | `http://localhost:11434/api/chat` | URL Chat API |
| `OLLAMA_MODEL` | `"llama3"` | Базовая модель |
| `OLLAMA_CUSTOM_MODEL` | `"alfons"` | Кастомная модель |
| `OLLAMA_TIMEOUT_SECS` | `60` | Таймаут запросов (сек) |
| `OLLAMA_INSTALL_SCRIPT` | `https://ollama.com/install.sh` | Скрипт установки |

### Константы yay

| Константа | Значение |
|-----------|----------|
| `YAY_INSTALL_DIR` | `/tmp/yay-install` |
| `YAY_AUR_URL` | `https://aur.archlinux.org/yay.git` |

### Прочие константы

| Константа | Значение | Описание |
|-----------|----------|----------|
| `CONFIG_APP_NAME` | `"alfons-assistant"` | Имя для confy |
| `MAX_CHAT_MESSAGES` | `100` | Лимит сообщений |
| `SETTINGS_PANEL_WIDTH` | `280.0` | Ширина панели настроек |

### Сообщения (`constants::messages`)

| Константа | Текст |
|-----------|-------|
| `WELCOME` | Система готова. Введите команду или задайте вопрос ИИ. |
| `CHAT_CLEARED` | История чата очищена. Чем могу помочь? |
| `PROCESSING` | Обработка... |
| `MODEL_CREATING` | Создаю кастомную модель 'alfons'... |
| `MODEL_CREATED` | [OK] Модель 'alfons' создана! Переключаю на неё. |
| `MODEL_EXISTS` | Модель 'alfons' уже существует. |
| `OLLAMA_INSTALLING` | Устанавливаю Ollama... |
| `OLLAMA_ALREADY` | Ollama уже установлена! |
| `OLLAMA_STARTING` | Запускаю сервис Ollama... |
| `OLLAMA_STARTED` | [OK] Сервис Ollama запущен! |
| `YAY_INSTALLING` | Устанавливаю yay... |
| `YAY_INSTALLED` | [OK] yay успешно установлен! |
| `YAY_ALREADY` | yay уже установлен! |

### Ошибки (`constants::errors`)

| Константа | Текст |
|-----------|-------|
| `OLLAMA_CONNECTION` | Ошибка связи с Ollama. Убедитесь, что сервис запущен. |
| `OLLAMA_PARSE` | Ошибка обработки ответа от Ollama. |
| `PACKAGE_NOT_FOUND` | Ничего не найдено. |
| `MODEL_CREATE_FAILED` | [X] Не удалось создать модель. |
| `MODEL_BASE_NOT_FOUND` | [X] Базовая модель llama3 не найдена. |
| `OLLAMA_INSTALL_FAILED` | [X] Не удалось установить Ollama. |
| `OLLAMA_START_FAILED` | [X] Не удалось запустить сервис Ollama. |
| `YAY_DEPS_FAILED` | [X] Не удалось установить зависимости для yay. |
| `YAY_CLONE_FAILED` | [X] Не удалось склонировать репозиторий yay. |
| `YAY_BUILD_FAILED` | [X] Не удалось собрать yay. |

---

## Сводная таблица функций

| Модуль | Функция | Описание |
|--------|---------|----------|
| **assistant_app** | `new()` | Создание приложения |
| | `check_ollama_periodic()` | Периодическая проверка Ollama |
| | `process_input()` | Обработка ввода пользователя |
| | `send_to_ai()` | Отправка в AI с историей |
| | `check_tasks()` | Проверка фоновых задач |
| | `process_ai_commands()` | Обработка `[CMD:...]` |
| | `clear_chat()` | Очистка чата |
| **chat** | `DialogState::new()` | Создание диалога |
| | `DialogState::show_search()` | Диалог поиска пакетов |
| | `DialogState::show_confirm()` | Диалог подтверждения |
| | `DialogState::hide()` | Скрытие диалога |
| | `ChatHistory::add_message()` | Добавить сообщение |
| | `ChatHistory::clear()` | Очистка истории |
| | `ChatHistory::messages()` | Итератор по сообщениям |
| | `ChatHistory::as_pairs()` | История как пары (sender, text) |
| | `TaskManager::new()` | Создание менеджера задач |
| | `TaskManager::execute()` | Запуск фоновой задачи |
| | `TaskManager::is_busy()` | Проверка занятости |
| | `InputHistory::push()` | Добавить в историю |
| | `InputHistory::up/down()` | Навигация по истории |
| | `InputHistory::reset()` | Сброс позиции |
| **config** | `Config::load()` | Загрузка настроек |
| | `Config::save()` | Сохранение настроек |
| | `Config::accent_color_egui()` | Конвертация цвета для egui |
| **ai** | `LocalAi::new()` | Создание AI клиента |
| | `LocalAi::set_model()` | Установка модели |
| | `LocalAi::get_model()` | Получение модели |
| | `LocalAi::generate()` | Генерация ответа с историей |
| | `check_ollama_status()` | Проверка сервиса Ollama |
| | `is_custom_model_exists()` | Проверка модели alfons |
| | `is_base_model_exists()` | Проверка модели llama3 |
| | `create_custom_model()` | Создание модели alfons |
| | `is_ollama_installed()` | Проверка установки Ollama |
| | `install_ollama()` | Установка Ollama |
| | `start_ollama_service()` | Запуск сервиса Ollama |
| | `ToolRegistry::new()` | Создание реестра инструментов |
| | `ToolRegistry::register()` | Регистрация инструмента |
| | `ToolRegistry::execute()` | Выполнение инструмента |
| | `ToolRegistry::generate_system_prompt()` | Генерация промпта |
| **commands** | `process_command()` | Главный обработчик команд |
| | `process_basic_command()` | Базовые команды |
| | `process_package_command()` | Пакетный менеджер |
| | `process_system_command()` | Системные команды |
| | `process_guide_command()` | Гайды |
| | `search_packages()` | Поиск пакетов |
| | `install_package()` | Установка пакета |
| | `remove_package()` | Удаление пакета |
| | `update_system()` | Обновление системы |
| | `is_yay_installed()` | Проверка yay |
| | `install_yay()` | Установка yay |
| | `execute_shutdown()` | Выключение ПК |
| | `execute_reboot()` | Перезагрузка ПК |
| **guides** | `GuideRegistry::new()` | Создание с гайдами |
| | `GuideRegistry::register()` | Регистрация гайда |
| | `GuideRegistry::get()` | Получить гайд по ID |
| | `GuideRegistry::search()` | Поиск гайдов |
| | `GuideRegistry::format_list()` | Список гайдов |
| | `Guide::format()` | Форматирование гайда |
| **ui** | `render()` | Главный рендеринг |
| | `handle_hotkeys()` | Горячие клавиши |
| | `render_header()` | Шапка приложения |
| | `render_settings()` | Панель настроек |
| | `render_chat()` | Область чата |
| | `render_input()` | Поле ввода |
| | `render_message()` | Пузырь сообщения |
| | `dialogs::render()` | Модальный диалог |
| **desktop** | `DesktopEnvironment::detect()` | Определение DE |
| | `terminal_priority()` | Приоритет терминалов |
| | `preferred_terminal()` | Предпочтительный терминал |
| | `name()` | Название DE |
| | `DeStyles::for_de()` | Стили для DE |
| | `run_in_terminal()` | Запуск команды в терминале |
| **installer** | `install()` | Установка в систему |
| | `uninstall()` | Удаление из системы |
| | `is_installed()` | Проверка установки |
| | `get_installed_path()` | Путь к бинарнику |
| | `is_local_bin_in_path()` | Проверка PATH |
| | `get_path_export_command()` | Команда для PATH |
| **command_log** | `log_command()` | Логирование команд |
