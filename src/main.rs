use eframe::egui;
use std::process::Command;
use chrono::Local;

// Главная функция, точка входа в приложение
fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 750.0])  // Устанавливаем начальный размер окна
            .with_title("Alfons Assistant"),   // Заголовок окна
        ..Default::default()
    };
    
    // Запускаем приложение
    eframe::run_native(
        "Alfons AI",  // Имя приложения
        options,
        Box::new(|cc| Ok(Box::new(AssistantApp::new(cc)))),
    )
}

// Структура для конфигурации приложения (поддерживает сериализацию)
#[derive(serde::Deserialize, serde::Serialize)]
struct Config {
    assistant_name: String,  // Имя ассистента
    accent_color: [u8; 3],   // Цвет акцента в формате RGB
}

// Главная структура приложения
struct AssistantApp {
    input_text: String,                // Текст из поля ввода
    chat_history: Vec<(String, String)>, // История чата: (отправитель, сообщение)
    config: Config,                     // Конфигурация
    show_settings: bool,                // Показывать ли панель настроек
}

impl AssistantApp {
    // Конструктор приложения
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            input_text: String::new(),  // Поле ввода пустое
            chat_history: vec![
                // Начальное сообщение
                ("Альфонс".into(), "Система Arch Linux готова. Настройте меня в меню справа.".into())
            ],
            config: Config {
                assistant_name: "Альфонс".into(),
                accent_color: [61, 174, 233], // Синий цвет KDE
            },
            show_settings: false,  // Настройки скрыты по умолчанию
        }
    }

    // Метод обработки ввода пользователя
    fn execute(&mut self) {
        // Проверяем, не пустой ли ввод (после удаления пробелов)
        if self.input_text.trim().is_empty() { 
            return;  // Если пусто - выходим
        }
        
        // 1. СОХРАНЕНИЕ ВВОДА ПОЛЬЗОВАТЕЛЯ
        let input = self.input_text.clone();  // Копируем текст из поля ввода
        self.chat_history.push(("Вы".into(), input.clone()));  // Добавляем в историю
        
        // 2. ПОДГОТОВКА К ОБРАБОТКЕ
        let cmd = input.trim().to_lowercase();  // Приводим к нижнему регистру и обрезаем пробелы
        
        // 3. ОБРАБОТКА КОМАНД
        let response = if cmd.contains("привет") {
            // Ответ на приветствие
            format!("Привет! Я {}, твой ассистент на Arch Linux.", self.config.assistant_name)
        } 
        else if cmd == "очистить" || cmd == "очистить чат" {
            // Очистка истории чата
            self.chat_history.clear();
            self.input_text.clear();  // Очищаем поле ввода
            return;  // Выходим, не добавляя сообщение в историю
        } 
        else if cmd.starts_with("скажи ") {
            // Команда для повтора текста
            // Пример: "скажи Привет мир" → "Привет мир"
            
            // Обрезаем префикс "скажи " (10 символов)
            let message = input.trim()[10..].trim().to_string();
            
            if message.is_empty() {
                "Что именно сказать?".into()
            } else {
                message
            }
        } 
        else if cmd == "выключить пк" || cmd == "выключить компьютер" {
            // Команда выключения компьютера (для Arch Linux)
            match Command::new("shutdown").args(&["-h", "now"]).status() {
                Ok(_) => "Система выключается...".into(),
                Err(e) => format!("Ошибка при выключении: {}", e)
            }
        } 
        else if cmd == "перезагрузить" || cmd == "рестарт" {
            // Команда перезагрузки
            match Command::new("shutdown").args(&["-r", "now"]).status() {
                Ok(_) => "Система перезагружается...".into(),
                Err(e) => format!("Ошибка при перезагрузке: {}", e)
            }
        }
        else if cmd.starts_with("выполни ") {
            // Команда выполнения произвольной команды в shell
            let shell_cmd = input.trim()[10..].trim();
            if shell_cmd.is_empty() {
                "Какую команду выполнить?".into()
            } else {
                match Command::new("sh").args(&["-c", shell_cmd]).output() {
                    Ok(output) => {
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        
                        if !stdout.is_empty() {
                            format!("Результат:\n{}", stdout.trim())
                        } else if !stderr.is_empty() {
                            format!("Ошибка:\n{}", stderr.trim())
                        } else {
                            "Команда выполнена (нет вывода)".into()
                        }
                    }
                    Err(e) => format!("Ошибка выполнения: {}", e)
                }
            }
        } else if cmd == "время" || cmd == "который час" {
    // Вариант 1: Использование Local (местное время)
            let now = Local::now();
            let formatted_time = now.format("%H:%M:%S").to_string();
            format!("Текущее время: {}", formatted_time)
        }
        else {
            // Если команда не распознана
            format!("Вы ввели: '{}'. Я пока не знаю такой команды.", input.trim())
        };
        
        // 4. ДОБАВЛЕНИЕ ОТВЕТА АССИСТЕНТА
        self.chat_history.push((self.config.assistant_name.clone(), response));
        
        // 5. ОЧИСТКА ПОЛЯ ВВОДА
        self.input_text.clear();
    }
}

// Реализация интерфейса eframe
impl eframe::App for AssistantApp {
    // Основной метод обновления интерфейса (вызывается каждый кадр)
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Создаем цвет акцента из конфигурации
        let accent = egui::Color32::from_rgb(
            self.config.accent_color[0], 
            self.config.accent_color[1], 
            self.config.accent_color[2]
        );

        // Настраиваем глобальный стиль приложения
        let mut style: egui::Style = (*ctx.style()).clone();
        style.spacing.item_spacing = egui::vec2(12.0, 12.0);  // Расстояние между элементами
        style.visuals.widgets.inactive.rounding = 12.0.into(); // Закругление углов
        ctx.set_style(style);

        // ВЕРХНЯЯ ПАНЕЛЬ (шапка приложения)
        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.add_space(10.0);
                // Заголовок с именем ассистента
                ui.heading(egui::RichText::new(format!("{}", self.config.assistant_name.to_uppercase()))
                    .strong().color(accent).size(22.0));
                
                // Кнопка настроек справа
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(10.0);
                    if ui.button(egui::RichText::new("⚙").size(20.0)).clicked() {
                        self.show_settings = !self.show_settings;  // Переключаем видимость настроек
                    }
                });
            });
            ui.add_space(10.0);
        });

        // ПАНЕЛЬ НАСТРОЕК (справа, появляется/скрывается)
        if self.show_settings {
            egui::SidePanel::right("settings_panel").default_width(250.0).show(ctx, |ui| {
                ui.add_space(20.0);
                ui.heading("Настройки");
                ui.separator();  // Разделитель
                
                // Настройка имени ассистента
                ui.label("Имя ассистента:");
                ui.text_edit_singleline(&mut self.config.assistant_name);
                
                ui.add_space(10.0);
                
                // Настройка цвета темы
                ui.label("Цвет темы:");
                ui.color_edit_button_srgb(&mut self.config.accent_color);
            });
        }

        // НИЖНЯЯ ПАНЕЛЬ (поле ввода и кнопка отправки)
        egui::TopBottomPanel::bottom("input_area")
            .frame(egui::Frame::none().inner_margin(egui::Margin {
                left: 20.0,
                right: 20.0,
                top: 15.0,
                bottom: 30.0,
            }))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // Поле ввода текста
                    let text_edit = ui.add_sized(
                        [ui.available_width() - 130.0, 45.0],  // Размер: доступная ширина минус место для кнопки
                        egui::TextEdit::singleline(&mut self.input_text)
                            .margin(egui::vec2(15.0, 11.0))  // Внутренние отступы
                            .hint_text("Напишите Асистенту...")  // Подсказка в пустом поле
                    );

                    ui.add_space(10.0);

                    // Кнопка отправки
                    let btn = egui::Button::new(egui::RichText::new("ОТПРАВИТЬ").strong())
                        .fill(accent)
                        .min_size(egui::vec2(110.0, 45.0));

                    // Обработка клика по кнопке или нажатия Enter
                    if ui.add(btn).clicked() || 
                       (text_edit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) {
                        self.execute();  // Выполняем команду
                        text_edit.request_focus();  // Возвращаем фокус в поле ввода
                    }
                });
            });
        
        // ЦЕНТРАЛЬНАЯ ПАНЕЛЬ (история чата)
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])  // Не уменьшаем область при маленьком контенте
                .stick_to_bottom(true)     // Прокручиваем вниз при новых сообщениях
                .show(ui, |ui| {
                    ui.add_space(15.0);
                    
                    // Отображаем все сообщения из истории
                    for (sender, text) in &self.chat_history {
                        let is_me = sender == "Вы";
                        
                        // Выбираем выравнивание: для пользователя - справа, для ассистента - слева
                        let layout = if is_me {
                            egui::Layout::right_to_left(egui::Align::TOP)
                        } else {
                            egui::Layout::left_to_right(egui::Align::TOP)
                        };

                        ui.with_layout(layout, |ui| {
                            // Ограничиваем ширину сообщения 70% от доступной
                            let max_width = ui.available_width() * 0.7;
                            
                            ui.scope(|ui| {
                                // Цвет фона сообщения
                                let frame_color = if is_me {
                                    egui::Color32::from_rgb(40, 80, 120)  // Синий для пользователя
                                } else {
                                    egui::Color32::from_gray(40)  // Темно-серый для ассистента
                                };

                                // Цвет границы сообщения
                                let stroke_color = if is_me {
                                    egui::Color32::from_rgb(60, 120, 180)
                                } else {
                                    accent.gamma_multiply(0.3)  // Более светлый оттенок акцентного цвета
                                };

                                // Отрисовываем "облачко" сообщения
                                egui::Frame::group(ui.style())
                                    .fill(frame_color)
                                    .stroke(egui::Stroke::new(1.0, stroke_color))
                                    .rounding(egui::Rounding {
                                        nw: 15.0,  // Верхний левый угол
                                        ne: 15.0,  // Верхний правый угол
                                        sw: if is_me { 15.0 } else { 2.0 },  // Нижний левый (острый для бота)
                                        se: if is_me { 2.0 } else { 15.0 },  // Нижний правый (острый для пользователя)
                                    })
                                    .inner_margin(12.0)  // Внутренние отступы
                                    .show(ui, |ui| {
                                        ui.set_max_width(max_width);  // Ограничиваем ширину
                                        ui.vertical(|ui| {
                                            // Имя отправителя
                                            ui.label(egui::RichText::new(sender)
                                                .strong()
                                                .color(if is_me { egui::Color32::LIGHT_BLUE } else { accent })
                                                .size(12.0));
                                            
                                            ui.add_space(2.0);
                                            
                                            // Текст сообщения
                                            ui.label(egui::RichText::new(text)
                                                .color(egui::Color32::WHITE)
                                                .size(15.0));
                                        });
                                    });
                            });
                        });
                        ui.add_space(8.0);  // Отступ между сообщениями
                    }
                });
        });
    }
}