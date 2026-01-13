
use super::super::chat::ChatMessage;
use eframe::egui;

/// Отрисовка пузыря сообщения с учетом отправителя
pub fn render_message_bubble(
    ui: &mut egui::Ui, 
    message: &ChatMessage, 
    accent_color: egui::Color32
) {
    let is_user = message.sender == "Вы";
    
    // Выбор стороны: пользователь справа, ассистент слева
    let layout = if is_user {
        egui::Layout::right_to_left(egui::Align::TOP)
    } else {
        egui::Layout::left_to_right(egui::Align::TOP)
    };
    

    ui.with_layout(layout, |ui| {
        // Ограничение ширины сообщения (70% от экрана)
        let max_width = ui.available_width() * 0.7;
        

        ui.scope(|ui| {
            // Определение цветовой схемы (темно-синий для пользователя, серый для ассистента)
            let frame_color = if is_user {
                egui::Color32::from_rgb(40, 80, 120) 
            } else {
                egui::Color32::from_gray(40)
            };
            
            // Цвет границы: мягкий синий или приглушенный акцентный цвет
            let stroke_color = if is_user {
                egui::Color32::from_rgb(60, 120, 180)
            } else {
                accent_color.gamma_multiply(0.3)
            };
            
            // Настройка контейнера (пузыря)
            egui::Frame::group(ui.style())
                .fill(frame_color)
                .stroke(egui::Stroke::new(1.0, stroke_color))
                .rounding(egui::Rounding {
                    nw: 15.0,
                    ne: 15.0,
                    // Создаем "хвостик" сообщения через разный радиус скругления
                    sw: if is_user { 15.0 } else { 2.0 },
                    se: if is_user { 2.0 } else { 15.0 },
                })
                .inner_margin(12.0)

                .show(ui, |ui| {
                    ui.set_max_width(max_width);

                    ui.vertical(|ui| {
                        // Отображение имени отправителя над текстом
                        ui.label(
                            egui::RichText::new(&message.sender)
                                .strong()
                                .color(if is_user {
                                    egui::Color32::LIGHT_BLUE
                                } else {
                                    accent_color
                                })
                                .size(12.0),
                        );
                        
                        ui.add_space(2.0);
                        
                        // Вывод основного содержимого (поддержка многострочности)
                        for line in message.text.lines() {
                            ui.label(
                                egui::RichText::new(line)
                                    .color(egui::Color32::WHITE)
                                    .size(15.0),
                            );
                        }
                    });
                });
        });
    });
}
