// src/main.rs

use eframe::egui;

use crate::app::PendulumApp;

mod app;
mod physics;
mod plots;
mod ui;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(1360.0, 820.0)),
        min_window_size: Some(egui::vec2(780.0, 560.0)),
        ..Default::default()
    };
    eframe::run_native(
        "Pendulum — Adaptive Improved Alignment",
        options,
        Box::new(|_cc| Box::new(PendulumApp::default())),
    )
}