//! Модальные диалоговые окна

use super::super::chat::{BackgroundTask, DialogType};
use super::super::AssistantApp;
use eframe::egui;

/// Отрисовка модального диалога
pub fn render(ctx: &egui::Context, app: &mut AssistantApp, accent: egui::Color32) {
    // Затемнение фона на нижнем слое
    let screen_rect = ctx.screen_rect();
    let overlay_painter = ctx.layer_painter(egui::LayerId::new(
        egui::Order::Middle,
        egui::Id::new("dialog_overlay"),
    ));
    overlay_painter.rect_filled(screen_rect, 0.0, egui::Color32::from_black_alpha(160));

    // Диалог как Area на слое Foreground (выше затемнения)
    egui::Area::new(egui::Id::new("dialog_window"))
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, -100.0))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            egui::Frame::window(&ctx.style()).show(ui, |ui| {
                ui.set_min_width(400.0);
                ui.set_max_width(500.0);

                // Заголовок
                ui.heading(&app.dialog.title);
                ui.separator();

                ui.vertical_centered(|ui| {
                    ui.add_space(10.0);

                    // Сообщение
                    for line in app.dialog.message.lines() {
                        ui.label(egui::RichText::new(line).size(15.0));
                    }

                    ui.add_space(15.0);

                    // Контент в зависимости от типа
                    match app.dialog.dialog_type {
                        DialogType::PackageSearch => {
                            ui.horizontal(|ui| {
                                ui.label("Пакет:");
                                let input = ui.add(
                                    egui::TextEdit::singleline(&mut app.dialog.input)
                                        .hint_text("название...")
                                        .desired_width(200.0),
                                );
                                if app.dialog.visible {
                                    input.request_focus();
                                }
                            });
                        }
                        DialogType::Confirmation
                            if !app.dialog.package.is_empty()
                                && !app.dialog.package.starts_with("__") =>
                        {
                            ui.label(
                                egui::RichText::new(&app.dialog.package)
                                    .strong()
                                    .color(accent),
                            );
                        }
                        _ => {}
                    }

                    ui.add_space(20.0);

                    // Кнопки
                    ui.horizontal(|ui| {
                        let btn_size = egui::vec2(100.0, 30.0);

                        // Отмена
                        if ui
                            .add_sized(btn_size, egui::Button::new("Отмена"))
                            .clicked()
                        {
                            app.dialog.hide();
                        }

                        // Основная кнопка
                        let action_text = match app.dialog.dialog_type {
                            DialogType::PackageSearch => "Найти",
                            DialogType::Confirmation => "Подтвердить",
                            DialogType::Info => "OK",
                        };

                        let action_btn =
                            egui::Button::new(egui::RichText::new(action_text).strong())
                                .fill(accent);

                        if ui.add_sized(btn_size, action_btn).clicked() {
                            handle_action(app);
                        }
                    });

                    ui.add_space(10.0);
                });
            });
        });
}

/// Обработка подтверждения
fn handle_action(app: &mut AssistantApp) {
    match app.dialog.dialog_type {
        DialogType::PackageSearch => {
            if !app.dialog.input.is_empty() {
                app.tasks
                    .execute(BackgroundTask::SearchPackages(app.dialog.input.clone()));
            }
        }
        DialogType::Confirmation => {
            let title = &app.dialog.title;
            let package = &app.dialog.package;

            if title.contains("Установка") && !package.is_empty() {
                app.tasks
                    .execute(BackgroundTask::InstallPackage(package.clone()));
            } else if title.contains("Удаление") && !package.is_empty() {
                app.tasks
                    .execute(BackgroundTask::RemovePackage(package.clone()));
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
