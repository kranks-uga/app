//! Модальные диалоговые окна

use super::super::chat::{BackgroundTask, DialogType};
use super::super::AssistantApp;
use eframe::egui;

/// Отрисовка модального диалога
pub fn render(ctx: &egui::Context, app: &mut AssistantApp, accent: egui::Color32) {
    // Затемнение фона
    let screen_rect = ctx.screen_rect();
    let overlay_painter = ctx.layer_painter(egui::LayerId::new(
        egui::Order::Middle,
        egui::Id::new("dialog_overlay"),
    ));
    overlay_painter.rect_filled(screen_rect, 0.0, egui::Color32::from_black_alpha(160));

    egui::Area::new(egui::Id::new("dialog_window"))
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, -60.0))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            egui::Frame::window(&ctx.style()).show(ui, |ui| {
                // Ширина диалога зависит от типа
                if matches!(app.dialog.dialog_type, DialogType::PackageResults) {
                    ui.set_min_width(560.0);
                    ui.set_max_width(660.0);
                } else {
                    ui.set_min_width(400.0);
                    ui.set_max_width(500.0);
                }

                ui.heading(&app.dialog.title);
                ui.separator();
                ui.add_space(8.0);

                match app.dialog.dialog_type {
                    DialogType::PackageResults => {
                        render_package_results(ui, app, accent);
                    }
                    _ => {
                        render_standard(ui, app, accent);
                    }
                }

                ui.add_space(8.0);
            });
        });
}

/// Диалог с результатами поиска пакетов
fn render_package_results(ui: &mut egui::Ui, app: &mut AssistantApp, accent: egui::Color32) {
    // Собираем данные заранее, чтобы не удерживать borrow на app.dialog
    let results: Vec<(String, String, String, String)> = app
        .dialog
        .search_results
        .iter()
        .map(|p| {
            (
                p.name.clone(),
                p.version.clone(),
                p.repo.clone(),
                p.description.clone(),
            )
        })
        .collect();

    let mut install_pkg: Option<String> = None;

    egui::ScrollArea::vertical()
        .max_height(380.0)
        .auto_shrink([false, true])
        .show(ui, |ui| {
            for (name, version, repo, desc) in &results {
                ui.horizontal(|ui| {
                    // Левая часть: имя + описание + репо
                    ui.vertical(|ui| {
                        ui.set_min_width(380.0);
                        ui.label(egui::RichText::new(name).strong().color(accent).size(14.0));
                        if !desc.is_empty() {
                            ui.label(egui::RichText::new(desc).weak().small());
                        }
                        ui.label(
                            egui::RichText::new(format!("{} · {}", repo, version))
                                .size(10.0)
                                .color(egui::Color32::GRAY),
                        );
                    });

                    // Правая часть: кнопка установки
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new("Установить").color(egui::Color32::WHITE),
                                )
                                .fill(accent),
                            )
                            .clicked()
                        {
                            install_pkg = Some(name.clone());
                        }
                    });
                });
                ui.add_space(4.0);
                ui.separator();
                ui.add_space(2.0);
            }
        });

    ui.add_space(10.0);
    ui.horizontal(|ui| {
        if ui
            .add_sized(
                egui::vec2(120.0, 30.0),
                egui::Button::new("Закрыть"),
            )
            .clicked()
        {
            app.dialog.hide();
        }
    });

    // Переходим к диалогу подтверждения установки выбранного пакета
    if let Some(pkg_name) = install_pkg {
        app.dialog.show_confirm(
            "Установка пакета",
            &format!("Установить '{}' через yay?", pkg_name),
            &pkg_name,
        );
    }
}

/// Стандартный диалог (поиск, подтверждение, инфо)
fn render_standard(ui: &mut egui::Ui, app: &mut AssistantApp, accent: egui::Color32) {
    ui.vertical_centered(|ui| {
        ui.add_space(6.0);

        // Сообщение
        for line in app.dialog.message.lines() {
            ui.label(egui::RichText::new(line).size(15.0));
        }

        ui.add_space(12.0);

        // Контент по типу
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

        ui.add_space(18.0);

        // Кнопки
        ui.horizontal(|ui| {
            let btn_size = egui::vec2(100.0, 30.0);

            if ui
                .add_sized(btn_size, egui::Button::new("Отмена"))
                .clicked()
            {
                app.dialog.hide();
            }

            let action_text = match app.dialog.dialog_type {
                DialogType::PackageSearch => "Найти",
                DialogType::Confirmation => "Подтвердить",
                DialogType::Info => "OK",
                DialogType::PackageResults => unreachable!(),
            };

            let action_btn =
                egui::Button::new(egui::RichText::new(action_text).strong()).fill(accent);

            if ui.add_sized(btn_size, action_btn).clicked() {
                handle_action(app);
            }
        });

        ui.add_space(8.0);
    });
}

/// Обработка подтверждения стандартных диалогов
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
        DialogType::PackageResults => {}
    }

    app.dialog.hide();
}
