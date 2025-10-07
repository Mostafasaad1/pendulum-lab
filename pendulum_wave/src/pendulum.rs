use eframe::egui;

#[derive(Clone)]
pub struct Pendulum {
    pub length: f32,
    pub angle: f32,
    pub angular_velocity: f32,
    pub period: f32,
    pub color: egui::Color32,
}