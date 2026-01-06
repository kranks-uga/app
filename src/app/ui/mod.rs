// src/app/ui/mod.rs

pub mod dialogs;
pub mod widgets;

use super::AssistantApp;
use eframe::egui;

/// Основная функция сборки интерфейса
pub fn render_ui(ctx: &egui::Context, app: &mut AssistantApp) {
    let accent_color = app.config.accent_color_egui();
    
    // 1. СНАЧАЛА ВЕРХ (Top)
    render_header(ctx, app, accent_color);
    
    // 2. ЗАТЕМ ПРАВАЯ ПАНЕЛЬ (Side)
    if app.show_settings {
        render_settings_panel(ctx, app, accent_color);
    }
    
    // 3. ЗАТЕМ НИЖНЯЯ ПАНЕЛЬ (Bottom)
    // Важно: вызываем до CentralPanel, чтобы занять место внизу
    render_input_panel(ctx, app, accent_color);
    
    // 4. ЦЕНТРАЛЬНАЯ ПАНЕЛЬ (Central)
    // Занимает всё оставшееся пространство
    render_chat_panel(ctx, app, accent_color);
    
    // 5. ДИАЛОГОВЫЕ ОКНА (Поверх всего)
    if app.show_dialog {
        dialogs::render_dialog(ctx, app, accent_color);
    }
}

/// Отрисовывает верхнюю панель с заголовком
fn render_header(ctx: &egui::Context, app: &mut AssistantApp, accent_color: egui::Color32) {
    egui::TopBottomPanel::top("header").show(ctx, |ui| {
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            ui.add_space(10.0);
            ui.heading(
                egui::RichText::new(app.config.assistant_name.to_uppercase())
                    .strong()
                    .color(accent_color)
                    .size(22.0),
            );
            
            if app.task_manager.is_processing() {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(10.0);
                    ui.label(egui::RichText::new("⏳ Обработка...").color(egui::Color32::YELLOW));
                });
            }
            
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

/// Отрисовывает панель настроек (справа)
fn render_settings_panel(ctx: &egui::Context, app: &mut AssistantApp, _accent_color: egui::Color32) {
    egui::SidePanel::right("settings_panel")
        .default_width(250.0)
        .show(ctx, |ui| {
            ui.add_space(20.0);
            ui.heading("Настройки");
            ui.separator();
            ui.label("Имя ассистента:");
            ui.text_edit_singleline(&mut app.config.assistant_name);
            ui.add_space(10.0);
            ui.label("Цвет темы:");
            ui.color_edit_button_srgb(&mut app.config.accent_color);
            ui.add_space(20.0);
            ui.separator();
            if ui.button("Проверить наличие yay").clicked() {
                app.task_manager.execute_task(super::chat::BackgroundTask::CheckYay);
            }
        });
}

/// Отрисовывает панель с историей чата
fn render_chat_panel(ctx: &egui::Context, app: &mut AssistantApp, accent_color: egui::Color32) {
    egui::CentralPanel::default().show(ctx, |ui| {
        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .stick_to_bottom(true)
            .show(ui, |ui| {
                ui.add_space(15.0);
                for message in app.chat_history.messages() {
                    widgets::render_message_bubble(ui, message, accent_color);
                    ui.add_space(8.0);
                }
            });
    });
}

/// Отрисовывает панель ввода сообщений
fn render_input_panel(ctx: &egui::Context, app: &mut AssistantApp, accent_color: egui::Color32) {
    egui::TopBottomPanel::bottom("input_area")
        .frame(egui::Frame::none().inner_margin(egui::Margin {
            left: 20.0, right: 20.0, top: 15.0, bottom: 30.0,
        }))
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
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

                if ui.add(btn).clicked() || 
                   (text_edit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) {
                    app.process_input();
                    text_edit.request_focus();
                }
            });
        });
}
