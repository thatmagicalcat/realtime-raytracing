mod app;
mod renderer;

use eframe::egui;

mod program;
use program::Program;

fn main() {
    let name = "Realtime Raytracing";
    eframe::run_native(
        name,
        eframe::NativeOptions {
            vsync: true,
            viewport: egui::ViewportBuilder {
                title: Some(name.to_string()),
                inner_size: Some(egui::vec2(900.0, 600.0)),
                ..Default::default()
            },
            ..Default::default()
        },
        Box::new(|cc| Ok(Box::new(app::Application::new(cc)))),
    )
    .unwrap();
}
