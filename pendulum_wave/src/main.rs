use eframe::egui;
use anyhow::Result;

mod app;
mod simulation;
mod pendulum;

use app::PendulumWaveApp;

fn main() -> Result<()> {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(1200.0, 800.0)),
        resizable: false,
        vsync: true,
        multisampling: 0,
        ..Default::default()
    };
    
    eframe::run_native(
        "Pendulum Wave Simulation - 2.5D",
        options,
        Box::new(|_cc| Box::new(PendulumWaveApp::default())),
    ).map_err(|e| anyhow::anyhow!("Failed to run app: {}", e))
}