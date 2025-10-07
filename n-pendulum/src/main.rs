mod pendulum;
mod solver;
mod gui;

fn main() {
    let options = eframe::NativeOptions {
        // Set the initial window size here:
        initial_window_size: Some(egui::Vec2::new(1200.0, 800.0)), // Make it bigger
        min_window_size: Some(egui::Vec2::new(800.0, 600.0)),      // Optional minimum size
        ..Default::default()
    };
    
    eframe::run_native("N-Pendulum (modular)", options, Box::new(|_cc| Box::new(gui::NPendulumApp::default()))).unwrap();
}
