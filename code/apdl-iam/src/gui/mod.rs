//! 图形用户界面模块
//!
//! 基于egui/eframe的GUI应用程序

use eframe::egui;

#[derive(Default)]
pub struct GuiApp;

impl GuiApp {
    pub fn new() -> Self {
        Self
    }
}

impl eframe::App for GuiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("APDL Interaction Access Module");
            ui.label("Welcome to the APDL GUI interface!");

            if ui.button("Click me").clicked() {
                println!("Button clicked!");
            }
        });
    }
}
