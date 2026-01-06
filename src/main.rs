mod app;

use app::AssistantApp;
use eframe::egui;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 750.0])
            .with_title("Alfons Assistant"),
        ..Default::default()
    };
    
    eframe::run_native(
        "Alfons AI",
        options,
        Box::new(|cc| Box::new(AssistantApp::new(cc))),
    )
}