//! Графический интерфейс

pub mod dialogs;
pub mod widgets;

use super::AssistantApp;
use super::chat::BackgroundTask;
use super::commands::package::is_yay_installed;
use super::constants::{APP_NAME, APP_VERSION, SETTINGS_PANEL_WIDTH, messages};
use eframe::egui;

/// Главная функция рендеринга
pub fn render(ctx: &egui::Context, app: &mut AssistantApp) {
    let accent = app.config.accent_color_egui();

    render_header(ctx, app, accent);

    if app.show_settings {
        render_settings(ctx, app, accent);
    }

    render_input(ctx, app, accent);
    render_chat(ctx, app, accent);

    if app.dialog.visible {
        dialogs::render(ctx, app, accent);
    }
}

/// Шапка приложения
fn render_header(ctx: &egui::Context, app: &mut AssistantApp, accent: egui::Color32) {
    egui::TopBottomPanel::top("header").show(ctx, |ui| {
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            ui.add_space(10.0);

            // Название
            ui.heading(
                egui::RichText::new(app.config.assistant_name.to_uppercase())
                    .strong()
                    .color(accent)
                    .size(22.0),
            );

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(10.0);

                // Кнопка настроек
                if ui.button(egui::RichText::new("⚙").size(20.0)).clicked() {
                    app.show_settings = !app.show_settings;
                }

                // Индикатор загрузки
                if app.tasks.is_busy() {
                    ui.label(
                        egui::RichText::new(messages::PROCESSING)
                            .color(egui::Color32::YELLOW),
                    );
                }
            });
        });
        ui.add_space(10.0);
    });
}

/// Панель настроек
fn render_settings(ctx: &egui::Context, app: &mut AssistantApp, accent: egui::Color32) {
    egui::SidePanel::right("settings")
        .default_width(SETTINGS_PANEL_WIDTH)
        .show(ctx, |ui| {
            ui.add_space(20.0);
            ui.heading("Настройки");
            ui.separator();

            let mut changed = false;

            // Персонализация
            ui.add_space(10.0);
            ui.label(egui::RichText::new("Персонализация").strong());
            ui.add_space(5.0);

            ui.label("Имя ассистента:");
            changed |= ui.text_edit_singleline(&mut app.config.assistant_name).changed();

            ui.add_space(10.0);
            ui.label("Цвет темы:");
            changed |= ui.color_edit_button_srgb(&mut app.config.accent_color).changed();

            if changed {
                app.config.save();
            }

            // Чат
            ui.add_space(20.0);
            ui.separator();
            ui.add_space(10.0);
            ui.label(egui::RichText::new("Чат").strong());
            ui.add_space(5.0);

            if ui.button(egui::RichText::new("🗑 Очистить чат").color(egui::Color32::LIGHT_RED)).clicked() {
                app.clear_chat();
            }

            // Пакетный менеджер
            ui.add_space(20.0);
            ui.separator();
            ui.add_space(10.0);
            ui.label(egui::RichText::new("Пакетный менеджер").strong());
            ui.add_space(5.0);

            let yay_ok = is_yay_installed();

            if yay_ok {
                ui.label(egui::RichText::new("✓ yay установлен").color(egui::Color32::LIGHT_GREEN));
            } else {
                ui.label(egui::RichText::new("✗ yay не найден").color(egui::Color32::LIGHT_RED));
            }

            ui.add_space(5.0);
            ui.horizontal(|ui| {
                if ui.button("🔍 Проверить").clicked() {
                    app.tasks.execute(BackgroundTask::CheckYay);
                }
                if !yay_ok && ui.button(egui::RichText::new("📦 Установить").color(accent)).clicked() {
                    app.tasks.execute(BackgroundTask::InstallYay);
                    app.chat.add_message("Система", messages::YAY_INSTALLING);
                }
            });

            // О программе
            ui.add_space(20.0);
            ui.separator();
            ui.add_space(10.0);
            ui.label(egui::RichText::new("О программе").strong());
            ui.add_space(5.0);
            ui.label(format!("{} — помощник для Arch Linux", APP_NAME));
            ui.label(egui::RichText::new(format!("v{}", APP_VERSION)).weak());
        });
}

/// Область чата
fn render_chat(ctx: &egui::Context, app: &mut AssistantApp, accent: egui::Color32) {
    egui::CentralPanel::default().show(ctx, |ui| {
        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .stick_to_bottom(true)
            .show(ui, |ui| {
                ui.add_space(10.0);
                for msg in app.chat.messages() {
                    widgets::render_message(ui, msg, accent);
                    ui.add_space(8.0);
                }
                ui.add_space(10.0);
            });
    });
}

/// Поле ввода
fn render_input(ctx: &egui::Context, app: &mut AssistantApp, accent: egui::Color32) {
    egui::TopBottomPanel::bottom("input")
        .frame(egui::Frame::none().inner_margin(egui::Margin {
            left: 20.0,
            right: 20.0,
            top: 15.0,
            bottom: 30.0,
        }))
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                let input = ui.add_sized(
                    [ui.available_width() - 130.0, 45.0],
                    egui::TextEdit::singleline(&mut app.input_text)
                        .margin(egui::vec2(15.0, 11.0))
                        .hint_text("Введите команду..."),
                );

                ui.add_space(10.0);

                let btn = egui::Button::new(egui::RichText::new("ОТПРАВИТЬ").strong())
                    .fill(accent)
                    .min_size(egui::vec2(110.0, 45.0));

                let enter = input.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));

                if ui.add(btn).clicked() || enter {
                    app.process_input();
                    input.request_focus();
                }
            });
        });
}
