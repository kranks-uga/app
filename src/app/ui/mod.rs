
// src/app/ui/mod.rs

pub mod dialogs; // Модуль модальных окон
pub mod widgets; // Модуль кастомных элементов (пузыри сообщений)

use super::AssistantApp;
use eframe::egui;

/// Главная функция сборки интерфейса (Layout менеджер)
pub fn render_ui(ctx: &egui::Context, app: &mut AssistantApp) {
    let accent_color = app.config.accent_color_egui();
    
    // 1. Верхняя панель (Header)
    render_header(ctx, app, accent_color);
    
    // 2. Боковая панель настроек (появляется по клику на шестеренку)
    if app.show_settings {
        render_settings_panel(ctx, app, accent_color);
    }
    
    // 3. Нижняя панель ввода (Bottom)
    // Отрисовывается до CentralPanel, чтобы зарезервировать место внизу
    render_input_panel(ctx, app, accent_color);
    
    // 4. Основная область чата (Central)
    // Занимает всё оставшееся пространство между Header и Input
    render_chat_panel(ctx, app, accent_color);
    
    // 5. Слой диалогов (Floating)
    // Рендерится поверх всех панелей при наличии активного события
    if app.show_dialog {
        dialogs::render_dialog(ctx, app, accent_color);
    }
}

/// Отрисовка шапки: заголовок, индикатор загрузки и кнопка настроек
fn render_header(ctx: &egui::Context, app: &mut AssistantApp, accent_color: egui::Color32) {

    egui::TopBottomPanel::top("header").show(ctx, |ui| {
        ui.add_space(10.0);

        ui.horizontal(|ui| {
            ui.add_space(10.0);
            ui.heading(
                egui::RichText::new(app.config.assistant_name.to_uppercase())
                    .strong().color(accent_color).size(22.0),
            );
            
            // Индикатор выполнения фоновых задач (например, установка пакетов)
            if app.task_manager.is_processing() {

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(10.0);
                    ui.label(egui::RichText::new("⏳ Обработка...").color(egui::Color32::YELLOW));
                });
            }
            
            // Кнопка переключения видимости панели настроек

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(10.0);
                if ui.button(egui::RichText::new("⚙").size(20.0)).clicked() {
                    app.show_settings = !app.show_settings;
                }
            });
        });
        ui.add_space(10.0);
    });
}

/// Конфигурация приложения: имя ассистента и выбор цвета темы
fn render_settings_panel(ctx: &egui::Context, app: &mut AssistantApp, _accent_color: egui::Color32) {
    egui::SidePanel::right("settings_panel")
        .default_width(250.0)

        .show(ctx, |ui| {
            ui.add_space(20.0);
            ui.heading("Настройки");
            ui.separator();

            let mut changed = false; // Флаг для отслеживания изменений

            ui.label("Имя ассистента:");
            if ui.text_edit_singleline(&mut app.config.assistant_name).changed() {
                changed = true;
            }

            ui.add_space(10.0);
            ui.label("Цвет темы:");
            if ui.color_edit_button_srgb(&mut app.config.accent_color).changed() {
                changed = true;
            }

            ui.vertical_centered_justified(|ui| {
                if ui.button(egui::RichText::new("🗑 Очистить чат").color(egui::Color32::LIGHT_RED)).clicked() {
                    app.clear_chat();
                }
            });

            // Если пользователь что-то поменял — сохраняем немедленно
            if changed {
                app.config.save();
            }

            ui.add_space(20.0);
            ui.separator();
            // Кнопка для системной проверки зависимостей (yay)
            if ui.button("Проверить наличие yay").clicked() {
                app.task_manager.execute_task(super::chat::BackgroundTask::CheckYay);
            }
        });
}

/// Область истории сообщений с автоматической прокруткой вниз
fn render_chat_panel(ctx: &egui::Context, app: &mut AssistantApp, accent_color: egui::Color32) {

    egui::CentralPanel::default().show(ctx, |ui| {
        egui::ScrollArea::vertical()
            .auto_shrink([false; 2]) // Запрет сжатия области, если сообщений мало
            .stick_to_bottom(true)   // Принудительная прокрутка к новым сообщениям

            .show(ui, |ui| {
                ui.add_space(15.0);
                for message in app.chat_history.messages() {
                    widgets::render_message_bubble(ui, message, accent_color);
                    ui.add_space(8.0);
                }
            });
    });
}

/// Поле ввода команды и кнопка отправки
fn render_input_panel(ctx: &egui::Context, app: &mut AssistantApp, accent_color: egui::Color32) {
    egui::TopBottomPanel::bottom("input_area")
        .frame(egui::Frame::none().inner_margin(egui::Margin {
            left: 20.0, right: 20.0, top: 15.0, bottom: 30.0,
        }))

        .show(ctx, |ui| {

            ui.horizontal(|ui| {
                // Основное текстовое поле
                let text_edit = ui.add_sized(
                    [ui.available_width() - 130.0, 45.0],
                    egui::TextEdit::singleline(&mut app.input_text)
                        .margin(egui::vec2(15.0, 11.0))
                        .hint_text("Введите команду..."),
                );

                ui.add_space(10.0);

                let btn = egui::Button::new(egui::RichText::new("ОТПРАВИТЬ").strong())
                    .fill(accent_color)
                    .min_size(egui::vec2(110.0, 45.0));

                // Логика отправки: клик или нажатие Enter

                if ui.add(btn).clicked() || 

                   (text_edit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) {
                    app.process_input();
                    text_edit.request_focus(); // Возвращаем курсор в поле после отправки
                }
            });
        });
}
