use eframe::egui;
use crate::simulation::PendulumSimulation;

#[derive(Default)]
pub struct PendulumWaveApp {
    pub simulation: PendulumSimulation,
    pub paused: bool,
    pub last_update: Option<f64>,
}

impl PendulumWaveApp {
    pub fn update_simulation(&mut self, current_time: f64) {
        if self.paused {
            return;
        }
        
        if let Some(last_time) = self.last_update {
            let delta_time = (current_time - last_time) as f32;
            self.simulation.update(delta_time);
        }
        self.last_update = Some(current_time);
    }
}

impl eframe::App for PendulumWaveApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let current_time = ctx.input(|i| i.time);
        self.update_simulation(current_time);
        
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Pendulum Wave Simulation - N Pendulums (2.5D View)");
            
            // Controls
            ui.horizontal(|ui| {
                if ui.button(if self.paused { "Resume" } else { "Pause" }).clicked() {
                    self.paused = !self.paused;
                }
                if ui.button("Reset").clicked() {
                    self.simulation.reset();
                    self.last_update = None;
                }
                ui.label("🎯 2.5D Perspective View with Depth & Shadows");
            });
            
            // Simulation info
            ui.label(format!("Time: {:.2}s", self.simulation.time));
            ui.label("Each pendulum has a slightly different length creating wave patterns");
            
            // Custom painting area
            let available_size = ui.available_size();
            let (rect, _response) = ui.allocate_exact_size(available_size, egui::Sense::hover());
            
            // Use the painter to draw our simulation
            let painter = ui.painter();
            self.simulation.draw(painter, rect);
        });
        
        // Only request repaint when not paused
        if !self.paused {
            ctx.request_repaint();
        }
    }
}