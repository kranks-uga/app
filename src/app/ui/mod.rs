//! Графический интерфейс

pub mod dialogs;
pub mod widgets;

use super::AssistantApp;
use super::chat::BackgroundTask;
use super::constants::{APP_NAME, APP_VERSION, SETTINGS_PANEL_WIDTH, messages};
use eframe::egui;
use std::sync::atomic::Ordering;

/// Главная функция рендеринга
pub fn render(ctx: &egui::Context, app: &mut AssistantApp) {
    let accent = app.config.accent_color_egui();

    // Горячие клавиши
    handle_hotkeys(ctx, app);

    render_header(ctx, app, accent);

    if app.show_settings {
        render_settings(ctx, app, accent);
    }

    render_input(ctx, app, accent);
    render_chat(ctx, app, accent);

    // Диалог с затемнением
    if app.dialog.visible {
        dialogs::render(ctx, app, accent);
    }
}

/// Обработка горячих клавиш
fn handle_hotkeys(ctx: &egui::Context, app: &mut AssistantApp) {
    ctx.input(|i| {
        // Ctrl+L — очистить чат
        if i.modifiers.ctrl && i.key_pressed(egui::Key::L) {
            app.clear_chat();
        }
        // Escape — закрыть настройки/диалог
        if i.key_pressed(egui::Key::Escape) {
            if app.dialog.visible {
                app.dialog.hide();
            } else if app.show_settings {
                app.show_settings = false;
            }
        }
    });
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

            // Индикатор Ollama
            let ollama_online = app.ollama_online.load(Ordering::SeqCst);
            let (status_text, status_color) = if ollama_online {
                ("[ON]", egui::Color32::LIGHT_GREEN)
            } else {
                ("[OFF]", egui::Color32::LIGHT_RED)
            };
            ui.label(egui::RichText::new(status_text).color(status_color).size(12.0))
                .on_hover_text(if ollama_online { "Ollama подключена" } else { "Ollama недоступна" });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(10.0);

                // Кнопка настроек
                if ui.button(egui::RichText::new("[=]").size(16.0)).clicked() {
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
            // Добавляем прокрутку для всего содержимого
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
            ui.add_space(20.0);
            ui.heading("Настройки");
            ui.separator();

            let mut changed = false;

            // Персонализация
            ui.add_space(10.0);
            ui.label(egui::RichText::new("Персонализация").strong());
            ui.add_space(5.0);

            ui.label("Цвет темы:");
            changed |= ui.color_edit_button_srgb(&mut app.config.accent_color).changed();

            // ИИ (Ollama)
            ui.add_space(20.0);
            ui.separator();
            ui.add_space(10.0);
            ui.label(egui::RichText::new("ИИ (Ollama)").strong());
            ui.add_space(5.0);

            // Статус
            let ollama_online = app.ollama_online.load(Ordering::SeqCst);
            if ollama_online {
                ui.label(egui::RichText::new("[OK] Подключено").color(egui::Color32::LIGHT_GREEN));
            } else {
                ui.label(egui::RichText::new("[X] Недоступно").color(egui::Color32::LIGHT_RED));
            }

            ui.add_space(5.0);
            ui.label("Модель:");
            let model_response = ui.add(
                egui::TextEdit::singleline(&mut app.config.ollama_model)
                    .hint_text("llama3")
                    .desired_width(150.0),
            );
            if model_response.changed() {
                app.ai.set_model(&app.config.ollama_model);
                changed = true;
            }

            ui.add_space(5.0);
            if ui.button("Проверить соединение").clicked() {
                let ollama_online = app.ollama_online.clone();
                tokio::spawn(async move {
                    let status = super::ai::local_provider::check_ollama_status().await;
                    ollama_online.store(status, Ordering::SeqCst);
                });
            }

            // Кастомная модель
            ui.add_space(10.0);
            ui.separator();
            ui.add_space(5.0);
            ui.label(egui::RichText::new("Кастомная модель").strong());
            ui.add_space(5.0);

            let model_exists = app.custom_model_exists.load(Ordering::SeqCst);
            if model_exists {
                ui.label(egui::RichText::new("[OK] Модель 'alfons' готова").color(egui::Color32::LIGHT_GREEN));
                ui.add_space(3.0);
                if ui.button("Использовать alfons").clicked() {
                    app.config.ollama_model = "alfons".to_string();
                    app.ai.set_model("alfons");
                    changed = true;
                }
            } else {
                ui.label(egui::RichText::new("Модель 'alfons' не создана").color(egui::Color32::GRAY));
                ui.add_space(3.0);
                ui.label(egui::RichText::new("Создаёт модель с оптимизированным промптом").weak().small());
                ui.add_space(3.0);
                if ui.button(egui::RichText::new("Создать модель alfons").color(accent)).clicked() {
                    app.tasks.execute(BackgroundTask::CreateCustomModel);
                    app.chat.add_message("Система", messages::MODEL_CREATING);
                    // Обновим статус после создания
                    let custom_model_exists = app.custom_model_exists.clone();
                    let ai = app.ai.clone();
                    std::thread::spawn(move || {
                        std::thread::sleep(std::time::Duration::from_secs(5));
                        let exists = super::ai::local_provider::is_custom_model_exists();
                        custom_model_exists.store(exists, Ordering::SeqCst);
                        if exists {
                            ai.set_model("alfons");
                        }
                    });
                }
            }

            // Чат
            ui.add_space(20.0);
            ui.separator();
            ui.add_space(10.0);
            ui.label(egui::RichText::new("Чат").strong());
            ui.add_space(5.0);

            if ui.button(egui::RichText::new("X Очистить чат").color(egui::Color32::LIGHT_RED)).clicked() {
                app.clear_chat();
            }
            ui.label(egui::RichText::new("Ctrl+L").weak().small());

            // Пакетный менеджер
            ui.add_space(20.0);
            ui.separator();
            ui.add_space(10.0);
            ui.label(egui::RichText::new("Пакетный менеджер").strong());
            ui.add_space(5.0);

            let yay_ok = app.yay_installed.load(Ordering::SeqCst);

            if yay_ok {
                ui.label(egui::RichText::new("[OK] yay установлен").color(egui::Color32::LIGHT_GREEN));
            } else {
                ui.label(egui::RichText::new("[X] yay не найден").color(egui::Color32::LIGHT_RED));
            }

            ui.add_space(5.0);
            ui.horizontal(|ui| {
                if ui.button("Проверить").clicked() {
                    // Обновляем кэш в фоне
                    let yay_installed = app.yay_installed.clone();
                    std::thread::spawn(move || {
                        let status = super::commands::package::is_yay_installed();
                        yay_installed.store(status, Ordering::SeqCst);
                    });
                }
                if !yay_ok && ui.button(egui::RichText::new("Установить yay").color(accent)).clicked() {
                    app.tasks.execute(BackgroundTask::InstallYay);
                    app.chat.add_message("Система", messages::YAY_INSTALLING);
                }
            });

            // Горячие клавиши
            ui.add_space(20.0);
            ui.separator();
            ui.add_space(10.0);
            ui.label(egui::RichText::new("Горячие клавиши").strong());
            ui.add_space(5.0);
            ui.label(egui::RichText::new("Ctrl+L — очистить чат").weak().small());
            ui.label(egui::RichText::new("Esc — закрыть панель").weak().small());
            ui.label(egui::RichText::new("↑/↓ — история команд").weak().small());

            // О программе
            ui.add_space(20.0);
            ui.separator();
            ui.add_space(10.0);
            ui.label(egui::RichText::new("О программе").strong());
            ui.add_space(5.0);
            ui.label(format!("{} — помощник для Arch Linux", APP_NAME));
            ui.label(egui::RichText::new(format!("v{}", APP_VERSION)).weak());

            // Установка в систему
            ui.add_space(20.0);
            ui.separator();
            ui.add_space(10.0);
            ui.label(egui::RichText::new("Установка").strong());
            ui.add_space(5.0);

            let is_installed = app.app_installed.load(Ordering::SeqCst);
            if is_installed {
                ui.label(egui::RichText::new("[OK] Установлено в систему").color(egui::Color32::LIGHT_GREEN));
                if let Some(path) = super::installer::get_installed_path() {
                    ui.label(egui::RichText::new(format!("{}", path.display())).weak().small());
                }
                ui.add_space(5.0);
                if ui.button(egui::RichText::new("Удалить из системы").color(egui::Color32::LIGHT_RED)).clicked() {
                    app.tasks.execute(BackgroundTask::UninstallFromSystem);
                    // Обновим статус после удаления
                    let app_installed = app.app_installed.clone();
                    std::thread::spawn(move || {
                        std::thread::sleep(std::time::Duration::from_millis(500));
                        app_installed.store(super::installer::is_installed(), Ordering::SeqCst);
                    });
                }
            } else {
                ui.label(egui::RichText::new("Не установлено").color(egui::Color32::GRAY));
                ui.add_space(3.0);
                ui.label(egui::RichText::new("Добавит ярлык в меню приложений").weak().small());
                ui.add_space(5.0);
                if ui.button(egui::RichText::new("Установить в систему").color(accent)).clicked() {
                    app.tasks.execute(BackgroundTask::InstallToSystem);
                    // Обновим статус после установки
                    let app_installed = app.app_installed.clone();
                    std::thread::spawn(move || {
                        std::thread::sleep(std::time::Duration::from_millis(500));
                        app_installed.store(super::installer::is_installed(), Ordering::SeqCst);
                    });
                }

                // Проверяем PATH
                if !super::installer::is_local_bin_in_path() {
                    ui.add_space(5.0);
                    ui.label(egui::RichText::new("~/.local/bin не в PATH").color(egui::Color32::YELLOW).small());
                    ui.label(egui::RichText::new("Добавьте в .bashrc/.zshrc:").weak().small());
                    ui.label(egui::RichText::new(super::installer::get_path_export_command()).weak().small().monospace());
                }
            }

            if changed {
                if let Err(e) = app.config.save() {
                    app.chat.add_message("Система", &e);
                }
            }

            ui.add_space(20.0);
            }); // конец ScrollArea
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

                // Сбрасываем позицию истории при ручном вводе текста
                if input.changed() {
                    app.input_history.reset();
                }

                // История команд (стрелки)
                if input.has_focus() {
                    ctx.input(|i| {
                        if i.key_pressed(egui::Key::ArrowUp) {
                            if let Some(prev) = app.input_history.up(&app.input_text) {
                                app.input_text = prev.to_string();
                            }
                        }
                        if i.key_pressed(egui::Key::ArrowDown) {
                            if let Some(next) = app.input_history.down() {
                                app.input_text = next.to_string();
                            }
                        }
                    });
                }

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
