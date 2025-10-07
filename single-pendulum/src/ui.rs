// src/ui.rs

use eframe::egui::{Align2, Color32, FontId, Painter, Pos2, Rect, Stroke};

use crate::app::PendulumApp;

pub fn draw_pendulum(app: &PendulumApp, painter: &Painter, rect: Rect) {
    let bg = if app.dark_mode {
        Color32::from_gray(18)
    } else {
        Color32::from_gray(245)
    };
    let text = if app.dark_mode {
        Color32::from_gray(220)
    } else {
        Color32::from_gray(30)
    };

    painter.rect_filled(rect, 6.0, bg);

    let center = rect.center();
    let scale = (rect.height() * 0.42).max(88.0); // slightly tighter than before
    let length_px = (app.length * scale).clamp(30.0, rect.height() * 0.85);

    let bob = Pos2::new(
        center.x + length_px * app.theta.sin(),
        center.y + length_px * app.theta.cos(),
    );
    let speed_ratio = (app.omega.abs() / 5.0).min(1.0);
    let rod_color = if speed_ratio > 0.5 {
        Color32::from_rgb(
            (255.0 * speed_ratio) as u8,
            (100.0 * (1.0 - speed_ratio)) as u8,
            100,
        )
    } else {
        Color32::from_rgb(
            100,
            (150.0 + 105.0 * speed_ratio) as u8,
            255,
        )
    };

    painter.line_segment([center, bob], Stroke::new(4.0, rod_color));
    painter.circle_filled(center, 6.0, Color32::from_gray(200));
    let bob_radius = 14.0 * (app.mass / 2.0).sqrt().clamp(0.6, 2.0);
    painter.circle_filled(bob, bob_radius, Color32::from_rgb(220, 70, 70));
    painter.circle_stroke(
        bob,
        bob_radius,
        Stroke::new(2.0, Color32::from_rgb(180, 60, 60)),
    );

    let (_p, _k, energy) = app.calculate_energy();
    let period = 2.0 * std::f32::consts::PI * (app.length / app.gravity).sqrt();
    let info = format!(
        "L:{:.2}m • θ:{:.1}° • ω:{:.1}°/s • T:{:.2}s • E:{:.2}J",
        app.length,
        app.theta.to_degrees(),
        app.omega.to_degrees(),
        period,
        energy
    );

    // tighter info placement (less vertical padding)
    painter.text(
        Pos2::new(rect.left() + 8.0, rect.top() + 8.0),
        Align2::LEFT_TOP,
        info,
        FontId::proportional(13.0),
        text,
    );
    painter.text(
        Pos2::new(rect.right() - 10.0, rect.top() + 8.0),
        Align2::RIGHT_TOP,
        format!("FPS:{:.1}", app.current_fps),
        FontId::monospace(11.0),
        text,
    );
}