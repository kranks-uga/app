use super::super::AssistantApp;
use eframe::egui;

/// Отрисовывает диалоговое окно
pub fn render_dialog(
    ctx: &egui::Context, 
    app: &mut AssistantApp, 
    _accent_color: egui::Color32
) {
    use super::super::chat::DialogType;

    // СОЗДАЕМ ЭФФЕКТ МОДАЛЬНОСТИ:
    // Рисуем полупрозрачный слой на весь экран перед окном, 
    // чтобы визуально отделить диалог от чата и строки ввода.
    let painter = ctx.layer_painter(egui::LayerId::new(egui::Order::Foreground, egui::Id::new("modal_darken")));
    painter.rect_filled(ctx.screen_rect(), 0.0, egui::Color32::from_black_alpha(160));
    
    egui::Window::new(&app.dialog_title)
        .collapsible(false)
        .resizable(false)
        // СМЕЩЕНИЕ: [0.0, -100.0] поднимает окно выше центра на 100 пикселей.
        // Это гарантирует, что оно не перекроет строку ввода даже на маленьких экранах.
        .anchor(egui::Align2::CENTER_CENTER, [0.0, -100.0])
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(10.0);
                
                for line in app.dialog_message.lines() {
                    ui.label(egui::RichText::new(line).size(15.0));
                }
                
                ui.add_space(15.0);
                
                match app.dialog_type {
                    DialogType::PackageSearch => {
                        ui.horizontal(|ui| {
                            ui.label("Пакет:");
                            let res = ui.add(
                                egui::TextEdit::singleline(&mut app.dialog_input)
                                    .hint_text("название...")
                                    .desired_width(200.0),
                            );
                            // Фокус на поле ввода сразу при открытии
                            if app.show_dialog { res.request_focus(); }
                        });
                    }
                    DialogType::Confirmation => {
                        if !app.dialog_package.is_empty() {
                            ui.label(egui::RichText::new(&app.dialog_package).strong().color(_accent_color));
                        }
                    }
                    _ => {}
                }
                
                ui.add_space(20.0);
                
                ui.horizontal(|ui| {
                    let btn_size = egui::vec2(100.0, 30.0);
                    
                    if ui.add_sized(btn_size, egui::Button::new("Отмена")).clicked() {
                        app.show_dialog = false;
                        app.dialog_input.clear();
                        app.dialog_package.clear();
                    }
                    
                    let action_text = match app.dialog_type {
                        DialogType::PackageSearch => "Найти",
                        DialogType::Confirmation => "Подтвердить",
                        _ => "OK",
                    };
                    
                    let action_btn = egui::Button::new(egui::RichText::new(action_text).strong())
                        .fill(_accent_color);
                        
                    if ui.add_sized(btn_size, action_btn).clicked() {
                        handle_dialog_action(app);
                    }
                });
                ui.add_space(10.0);
            });
        });
}

/// Логика нажатия кнопок в диалоге (без изменений)
fn handle_dialog_action(app: &mut AssistantApp) {
    use super::super::chat::{BackgroundTask, DialogType};
    
    match app.dialog_type {
        DialogType::PackageSearch => {
            if !app.dialog_input.is_empty() {
                app.task_manager.execute_task(BackgroundTask::SearchPackages(app.dialog_input.clone()));
                app.show_dialog = false;
                app.dialog_input.clear();
            }
        }
        DialogType::Confirmation => {
            // Если заголовок содержит нужные слова, запускаем задачи
            if app.dialog_title.contains("Установка") && !app.dialog_package.is_empty() {
                app.task_manager.execute_task(BackgroundTask::InstallPackage(app.dialog_package.clone()));
            } else if app.dialog_title.contains("Удаление") && !app.dialog_package.is_empty() {
                app.task_manager.execute_task(BackgroundTask::RemovePackage(app.dialog_package.clone()));
            } else if app.dialog_title.contains("Обновление") {
                app.task_manager.execute_task(BackgroundTask::UpdateSystem);
            }
            app.show_dialog = false;
            app.dialog_package.clear();
        }
        _ => app.show_dialog = false,
    }
}
