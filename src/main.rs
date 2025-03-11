mod app;
mod renderer;
mod camera;
mod state;
mod ray;
mod world;

use state::*;

use eframe::egui;

mod program;
use program::Program;

fn main() {
    let name = "Realtime Raytracing";
    eframe::run_native(
        name,
        eframe::NativeOptions {
            vsync: false,
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
