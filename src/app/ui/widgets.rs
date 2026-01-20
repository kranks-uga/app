//! Кастомные виджеты

use super::super::chat::ChatMessage;
use eframe::egui;

/// Пузырь сообщения в чате
pub fn render_message(ui: &mut egui::Ui, msg: &ChatMessage, accent: egui::Color32) {
    let is_user = msg.sender == "Вы";

    // Цвета
    let (bg, border, name_color) = if is_user {
        (
            egui::Color32::from_rgb(40, 80, 120),
            egui::Color32::from_rgb(60, 120, 180),
            egui::Color32::LIGHT_BLUE,
        )
    } else {
        (
            egui::Color32::from_gray(40),
            accent.gamma_multiply(0.3),
            accent,
        )
    };

    // Скругления
    let rounding = egui::Rounding {
        nw: 15.0,
        ne: 15.0,
        sw: if is_user { 15.0 } else { 2.0 },
        se: if is_user { 2.0 } else { 15.0 },
    };

    // Выравнивание
    let layout = if is_user {
        egui::Layout::right_to_left(egui::Align::TOP)
    } else {
        egui::Layout::left_to_right(egui::Align::TOP)
    };

    // Максимальная ширина пузыря - 70% от доступной ширины
    let max_bubble_width = ui.available_width() * 0.7;

    ui.with_layout(layout, |ui| {
        egui::Frame::none()
            .fill(bg)
            .stroke(egui::Stroke::new(1.0, border))
            .rounding(rounding)
            .inner_margin(12.0)
            .show(ui, |ui| {
                ui.set_max_width(max_bubble_width);

                // Заголовок: имя + время
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(&msg.sender)
                            .strong()
                            .color(name_color)
                            .size(12.0),
                    );
                    ui.add_space(10.0);
                    ui.label(
                        egui::RichText::new(msg.timestamp.format("%H:%M").to_string())
                            .weak()
                            .size(10.0),
                    );
                });

                ui.add_space(2.0);

                // Текст сообщения с переносом
                let text_response = ui.add(
                    egui::Label::new(
                        egui::RichText::new(&msg.text)
                            .color(egui::Color32::WHITE)
                            .size(15.0),
                    )
                    .wrap(true)
                    .sense(egui::Sense::click()),
                );

                // Копирование по клику
                if text_response.clicked() {
                    ui.output_mut(|o| o.copied_text = msg.text.clone());
                }
                text_response.on_hover_text("Нажмите чтобы скопировать");
            });
    });
}
