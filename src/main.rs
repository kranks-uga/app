use eframe::egui;
use std::process::Command;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 750.0])
            .with_title("Alfons OS Assistant"),
        ..Default::default()
    };
    
    eframe::run_native(
        "Alfons AI",
        options,
        Box::new(|cc| Ok(Box::new(AssistantApp::new(cc)))),
    )
}

#[derive(serde::Deserialize, serde::Serialize)]
struct Config {
    assistant_name: String,
    accent_color: [u8; 3],
}

struct AssistantApp {
    input_text: String,
    chat_history: Vec<(String, String)>,
    config: Config,
    show_settings: bool,
}

impl AssistantApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            input_text: String::new(),
            chat_history: vec![("Альфонс".into(), "Система Arch Linux готова. Настройте меня в меню справа.".into())],
            config: Config {
                assistant_name: "Альфонс".into(),
                accent_color: [61, 174, 233], // KDE Blue
            },
            show_settings: false,
        }
    }

    fn execute(&mut self) {
        if self.input_text.trim().is_empty() { return; }
        let input = self.input_text.clone();
        self.chat_history.push(("Вы".into(), input.clone()));
        
        // Логика (сокращенно для примера)
        let response = match input.to_lowercase().as_str() {
            "терминал" => { Command::new("konsole").spawn().ok(); "Открываю Konsole...".into() },
            _ => format!("Вы сказали: {}. Я готов к командам Arch.", input),
        };

        self.chat_history.push((self.config.assistant_name.clone(), response));
        self.input_text.clear();
    }
}

impl eframe::App for AssistantApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let accent = egui::Color32::from_rgb(self.config.accent_color[0], self.config.accent_color[1], self.config.accent_color[2]);

        // Глобальные настройки стиля
        let mut style: egui::Style = (*ctx.style()).clone();
        style.spacing.item_spacing = egui::vec2(12.0, 12.0);
        style.visuals.widgets.inactive.rounding = 12.0.into();
        ctx.set_style(style);

        // ВЕРХНЯЯ ПАНЕЛЬ
        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.add_space(10.0);
                ui.heading(egui::RichText::new(format!("{} OS", self.config.assistant_name.to_uppercase()))
                    .strong().color(accent).size(22.0));
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(10.0);
                    if ui.button(egui::RichText::new("⚙").size(20.0)).clicked() {
                        self.show_settings = !self.show_settings;
                    }
                });
            });
            ui.add_space(10.0);
        });

        // ПАНЕЛЬ НАСТРОЕК (СПРАВА)
        if self.show_settings {
            egui::SidePanel::right("settings_panel").default_width(250.0).show(ctx, |ui| {
                ui.add_space(20.0);
                ui.heading("Настройки");
                ui.separator();
                ui.label("Имя ассистента:");
                ui.text_edit_singleline(&mut self.config.assistant_name);
                ui.add_space(10.0);
                ui.label("Цвет темы:");
                ui.color_edit_button_srgb(&mut self.config.accent_color);
            });
        }

        // НИЖНЯЯ ПАНЕЛЬ ВВОДА (Теперь выше и с отступами)
        egui::TopBottomPanel::bottom("input_area")
            .frame(egui::Frame::none().inner_margin(egui::Margin {
                left: 20.0,
                right: 20.0,
                top: 15.0,
                bottom: 30.0, // Увеличили нижний отступ, чтобы поднять ввод выше
            }))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // Красивое поле ввода
                    let text_edit = ui.add_sized(
                        [ui.available_width() - 130.0, 45.0],
                        egui::TextEdit::singleline(&mut self.input_text)
                            .margin(egui::vec2(15.0, 11.0))
                            .hint_text("Напишите Альфонсу...")
                    );

                    ui.add_space(10.0);

                    // Стильная кнопка
                    let btn = egui::Button::new(egui::RichText::new("ОТПРАВИТЬ").strong())
                        .fill(accent)
                        .min_size(egui::vec2(110.0, 45.0));

                    if ui.add(btn).clicked() || (text_edit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) {
                        self.execute();
                        text_edit.request_focus();
                    }
                });
            });
        // ЦЕНТРАЛЬНАЯ ПАНЕЛЬ (ЧАТ)
egui::CentralPanel::default().show(ctx, |ui| {
    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .stick_to_bottom(true)
        .show(ui, |ui| {
            ui.add_space(15.0);
            
            for (sender, text) in &self.chat_history {
                let is_me = sender == "Вы";
                
                // Используем Layout, чтобы прижать сообщение к нужному краю
                let layout = if is_me {
                    egui::Layout::right_to_left(egui::Align::TOP)
                } else {
                    egui::Layout::left_to_right(egui::Align::TOP)
                };

                ui.with_layout(layout, |ui| {
                    // Ограничиваем максимальную ширину облачка сообщения (70% от ширины окна)
                    let max_width = ui.available_width() * 0.7;
                    
                    ui.scope(|ui| {
                        // Цвета для разных участников
                        let frame_color = if is_me {
                            egui::Color32::from_rgb(40, 80, 120) // Темно-синий для пользователя
                        } else {
                            egui::Color32::from_gray(40) // Темно-серый для Альфонса
                        };

                        let stroke_color = if is_me {
                            egui::Color32::from_rgb(60, 120, 180)
                        } else {
                            accent.gamma_multiply(0.3)
                        };

                        egui::Frame::group(ui.style())
                            .fill(frame_color)
                            .stroke(egui::Stroke::new(1.0, stroke_color))
                            .rounding(egui::Rounding {
                                nw: 15.0,
                                ne: 15.0,
                                sw: if is_me { 15.0 } else { 2.0 }, // Острый угол снизу слева для бота
                                se: if is_me { 2.0 } else { 15.0 }, // Острый угол снизу справа для профиля
                            })
                            .inner_margin(12.0)
                            .show(ui, |ui| {
                                ui.set_max_width(max_width); // Сообщение не будет шире 70% окна
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
                ui.add_space(8.0); // Расстояние между сообщениями
            }
        });
});

    }
}


