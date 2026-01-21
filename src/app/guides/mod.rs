use std::collections::HashMap;

/// Один шаг гайда
#[derive(Clone)]
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

/// Гайд по теме
#[derive(Clone)]
pub struct Guide {
    pub id: String,
    pub title: String,
    pub description: String,
    pub steps: Vec<GuideStep>,
    pub tags: Vec<String>,
}

impl Guide {
    pub fn new(id: &str, title: &str, description: &str) -> Self {
        Self {
            id: id.to_string(),
            title: title.to_string(),
            description: description.to_string(),
            steps: Vec::new(),
            tags: Vec::new(),
        }
    }

    pub fn add_step(mut self, step: GuideStep) -> Self {
        self.steps.push(step);
        self
    }

    pub fn add_tags(mut self, tags: &[&str]) -> Self {
        self.tags.extend(tags.iter().map(|t| t.to_string()));
        self
    }

    /// Форматирует гайд для вывода в чат
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
}

/// Контейнер всех гайдов
pub struct GuideRegistry {
    guides: HashMap<String, Guide>,
}

impl GuideRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            guides: HashMap::new(),
        };
        registry.register_default_guides();
        registry
    }

    /// Регистрирует гайд
    pub fn register(&mut self, guide: Guide) {
        self.guides.insert(guide.id.clone(), guide);
    }

    /// Получить гайд по ID
    pub fn get(&self, id: &str) -> Option<&Guide> {
        self.guides.get(id)
    }

    /// Поиск гайдов по ключевому слову (в названии, описании, тегах)
    pub fn search(&self, query: &str) -> Vec<&Guide> {
        let query_lower = query.to_lowercase();
        self.guides
            .values()
            .filter(|g| {
                g.title.to_lowercase().contains(&query_lower)
                    || g.description.to_lowercase().contains(&query_lower)
                    || g.tags.iter().any(|t| t.to_lowercase().contains(&query_lower))
                    || g.id.to_lowercase().contains(&query_lower)
            })
            .collect()
    }

    /// Список всех гайдов (ID и название)
    pub fn list(&self) -> Vec<(&str, &str)> {
        self.guides
            .values()
            .map(|g| (g.id.as_str(), g.title.as_str()))
            .collect()
    }

    /// Форматирует список гайдов для вывода
    pub fn format_list(&self) -> String {
        let mut output = String::from("📚 Доступные гайды:\n\n");

        let mut guides: Vec<_> = self.guides.values().collect();
        guides.sort_by(|a, b| a.title.cmp(&b.title));

        for guide in guides {
            output.push_str(&format!("• {} — {}\n", guide.id, guide.title));
        }

        output.push_str("\nИспользуйте: гайд <название>");
        output
    }

    /// Регистрирует базовые гайды
    fn register_default_guides(&mut self) {
        // Pacman
        self.register(
            Guide::new("pacman", "Основы Pacman", "Базовые команды пакетного менеджера Arch Linux")
                .add_tags(&["пакеты", "установка", "обновление", "packages"])
                .add_step(GuideStep::new("Обновить список пакетов и систему")
                    .with_command("sudo pacman -Syu")
                    .with_note("Рекомендуется делать перед установкой новых пакетов"))
                .add_step(GuideStep::new("Установить пакет")
                    .with_command("sudo pacman -S <пакет>"))
                .add_step(GuideStep::new("Удалить пакет")
                    .with_command("sudo pacman -R <пакет>"))
                .add_step(GuideStep::new("Удалить пакет с зависимостями")
                    .with_command("sudo pacman -Rns <пакет>")
                    .with_note("Удаляет также неиспользуемые зависимости и конфиги"))
                .add_step(GuideStep::new("Поиск пакета")
                    .with_command("pacman -Ss <запрос>"))
                .add_step(GuideStep::new("Информация о пакете")
                    .with_command("pacman -Si <пакет>"))
                .add_step(GuideStep::new("Список установленных пакетов")
                    .with_command("pacman -Q"))
                .add_step(GuideStep::new("Очистить кэш пакетов")
                    .with_command("sudo pacman -Sc")
                    .with_note("Удаляет старые версии из /var/cache/pacman/pkg"))
        );

        // AUR и yay
        self.register(
            Guide::new("aur", "Работа с AUR", "Arch User Repository — пользовательские пакеты")
                .add_tags(&["yay", "aur", "пользовательские", "репозиторий"])
                .add_step(GuideStep::new("Установить yay (AUR helper)")
                    .with_command("sudo pacman -S --needed git base-devel && git clone https://aur.archlinux.org/yay.git && cd yay && makepkg -si"))
                .add_step(GuideStep::new("Поиск в AUR")
                    .with_command("yay -Ss <запрос>")
                    .with_note("Ищет и в официальных репозиториях, и в AUR"))
                .add_step(GuideStep::new("Установить пакет из AUR")
                    .with_command("yay -S <пакет>"))
                .add_step(GuideStep::new("Обновить все пакеты (включая AUR)")
                    .with_command("yay -Syu"))
                .add_step(GuideStep::new("Показать статистику AUR пакетов")
                    .with_command("yay -Ps"))
        );

        // WiFi
        self.register(
            Guide::new("wifi", "Настройка WiFi", "Подключение к беспроводной сети")
                .add_tags(&["сеть", "интернет", "wireless", "network", "вайфай"])
                .add_step(GuideStep::new("Проверить сетевые интерфейсы")
                    .with_command("ip link")
                    .with_note("Найдите интерфейс wlan0 или похожий"))
                .add_step(GuideStep::new("Включить интерфейс")
                    .with_command("sudo ip link set wlan0 up"))
                .add_step(GuideStep::new("Сканировать доступные сети")
                    .with_command("sudo iwlist wlan0 scan | grep ESSID"))
                .add_step(GuideStep::new("Подключиться через iwctl (iwd)")
                    .with_command("iwctl station wlan0 connect <имя_сети>")
                    .with_note("Для WPA сетей запросит пароль"))
                .add_step(GuideStep::new("Или через NetworkManager")
                    .with_command("nmcli device wifi connect <имя_сети> password <пароль>"))
                .add_step(GuideStep::new("Проверить подключение")
                    .with_command("ping -c 3 archlinux.org"))
        );

        // Systemd
        self.register(
            Guide::new("systemd", "Управление сервисами", "Основы работы с systemd")
                .add_tags(&["сервисы", "службы", "services", "демоны"])
                .add_step(GuideStep::new("Статус сервиса")
                    .with_command("systemctl status <сервис>"))
                .add_step(GuideStep::new("Запустить сервис")
                    .with_command("sudo systemctl start <сервис>"))
                .add_step(GuideStep::new("Остановить сервис")
                    .with_command("sudo systemctl stop <сервис>"))
                .add_step(GuideStep::new("Перезапустить сервис")
                    .with_command("sudo systemctl restart <сервис>"))
                .add_step(GuideStep::new("Включить автозапуск")
                    .with_command("sudo systemctl enable <сервис>"))
                .add_step(GuideStep::new("Отключить автозапуск")
                    .with_command("sudo systemctl disable <сервис>"))
                .add_step(GuideStep::new("Список всех сервисов")
                    .with_command("systemctl list-units --type=service"))
                .add_step(GuideStep::new("Просмотр логов сервиса")
                    .with_command("journalctl -u <сервис> -f")
                    .with_note("-f для отслеживания в реальном времени"))
        );

        // Драйверы видеокарты
        self.register(
            Guide::new("gpu", "Драйверы видеокарты", "Установка драйверов для GPU")
                .add_tags(&["видеокарта", "драйверы", "nvidia", "amd", "intel", "графика"])
                .add_step(GuideStep::new("Определить видеокарту")
                    .with_command("lspci -v | grep -i vga"))
                .add_step(GuideStep::new("Для Intel")
                    .with_command("sudo pacman -S mesa intel-media-driver"))
                .add_step(GuideStep::new("Для AMD")
                    .with_command("sudo pacman -S mesa xf86-video-amdgpu vulkan-radeon"))
                .add_step(GuideStep::new("Для NVIDIA (проприетарный)")
                    .with_command("sudo pacman -S nvidia nvidia-utils nvidia-settings")
                    .with_note("После установки нужна перезагрузка"))
                .add_step(GuideStep::new("Для NVIDIA (открытый)")
                    .with_command("sudo pacman -S xf86-video-nouveau"))
                .add_step(GuideStep::new("Проверить драйвер")
                    .with_command("glxinfo | grep 'OpenGL renderer'"))
        );

        // Звук
        self.register(
            Guide::new("audio", "Настройка звука", "Pipewire и управление аудио")
                .add_tags(&["звук", "аудио", "pipewire", "pulseaudio", "sound"])
                .add_step(GuideStep::new("Установить Pipewire")
                    .with_command("sudo pacman -S pipewire pipewire-pulse pipewire-alsa wireplumber"))
                .add_step(GuideStep::new("Включить сервис")
                    .with_command("systemctl --user enable --now pipewire pipewire-pulse wireplumber"))
                .add_step(GuideStep::new("Установить графический микшер")
                    .with_command("sudo pacman -S pavucontrol"))
                .add_step(GuideStep::new("Проверить устройства вывода")
                    .with_command("wpctl status"))
                .add_step(GuideStep::new("Установить громкость")
                    .with_command("wpctl set-volume @DEFAULT_AUDIO_SINK@ 50%"))
                .add_step(GuideStep::new("Переключить устройство по умолчанию")
                    .with_command("wpctl set-default <ID>")
                    .with_note("ID можно узнать из wpctl status"))
        );

        // Локализация
        self.register(
            Guide::new("locale", "Локализация системы", "Настройка языка и раскладки")
                .add_tags(&["язык", "русский", "раскладка", "клавиатура", "локаль"])
                .add_step(GuideStep::new("Раскомментировать нужные локали")
                    .with_command("sudo nano /etc/locale.gen")
                    .with_note("Раскомментируйте en_US.UTF-8 и ru_RU.UTF-8"))
                .add_step(GuideStep::new("Сгенерировать локали")
                    .with_command("sudo locale-gen"))
                .add_step(GuideStep::new("Установить системную локаль")
                    .with_command("sudo localectl set-locale LANG=ru_RU.UTF-8"))
                .add_step(GuideStep::new("Настроить раскладку клавиатуры")
                    .with_command("sudo localectl set-x11-keymap us,ru pc105 , grp:alt_shift_toggle")
                    .with_note("Переключение по Alt+Shift"))
                .add_step(GuideStep::new("Проверить настройки")
                    .with_command("localectl status"))
        );

        // Timeshift / бэкапы
        self.register(
            Guide::new("backup", "Резервное копирование", "Создание бэкапов системы")
                .add_tags(&["бэкап", "backup", "timeshift", "снимки", "восстановление"])
                .add_step(GuideStep::new("Установить Timeshift")
                    .with_command("yay -S timeshift"))
                .add_step(GuideStep::new("Создать снимок системы")
                    .with_command("sudo timeshift --create --comments 'Мой бэкап'"))
                .add_step(GuideStep::new("Список снимков")
                    .with_command("sudo timeshift --list"))
                .add_step(GuideStep::new("Восстановить из снимка")
                    .with_command("sudo timeshift --restore")
                    .with_note("Выберите снимок интерактивно"))
                .add_step(GuideStep::new("Настроить автоматические снимки")
                    .with_command("sudo timeshift-gtk")
                    .with_note("Графический интерфейс для настройки"))
        );
    }
}

impl Default for GuideRegistry {
    fn default() -> Self {
        Self::new()
    }
}
