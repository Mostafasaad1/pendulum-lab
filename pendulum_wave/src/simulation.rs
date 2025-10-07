use eframe::egui;
use crate::pendulum::Pendulum;

pub struct PendulumSimulation {
    pub pendulums: Vec<Pendulum>,
    pub time: f32,
    // Precomputed values for performance
    pub total_width: f32,
    pub base_spacing: f32,
    pub depth_factors: Vec<f32>,
    pub spacing_offsets: Vec<f32>,
    pub wave_points: Vec<Vec<egui::Pos2>>,
    pub wave_rect: egui::Rect,
}

impl Default for PendulumSimulation {
    fn default() -> Self {
        let num_pendulums = 9;
        let mut pendulums = Vec::with_capacity(num_pendulums);
        let base_spacing = 120.0;
        
        // Precompute depth factors and spacing
        let mut depth_factors = Vec::with_capacity(num_pendulums);
        let mut spacing_offsets = Vec::with_capacity(num_pendulums);
        let mut total_width = 0.0;
        
        for i in 0..num_pendulums {
            let depth_factor = 1.0 - (i as f32 * 0.1);
            depth_factors.push(depth_factor);
            let spacing = base_spacing * depth_factor * depth_factor;
            spacing_offsets.push(total_width);
            total_width += spacing;
            
            // Base length increases with each pendulum
            let length = 1.0 + (i as f32) * 0.1;
            // Different periods create the wave effect
            let period = 2.0 + (i as f32) * 0.2;
            
            // Generate unique color for each pendulum
            let color = Self::generate_distinct_color(i, num_pendulums);
            
            pendulums.push(Pendulum {
                length,
                angle: std::f32::consts::FRAC_PI_2, 
                angular_velocity: 0.0,
                period,
                color,
            });
        }
        
        // Initialize wave points with dummy values (will be updated during first draw)
        let wave_points = vec![Vec::new(); num_pendulums];
        let wave_rect = egui::Rect::NOTHING;
        
        Self {
            pendulums,
            time: 0.0,
            total_width,
            base_spacing,
            depth_factors,
            spacing_offsets,
            wave_points,
            wave_rect,
        }
    }
}

impl PendulumSimulation {
    pub fn generate_distinct_color(index: usize, total: usize) -> egui::Color32 {
        // Distribute hues evenly
        let hue = index as f32 / total as f32;
        
        // Vary saturation to make colors more distinct
        let saturation = 0.7 + (index % 3) as f32 * 0.15;
        let value = 0.8 + (index % 2) as f32 * 0.15;
        
        // Convert HSV to RGB
        let h = hue * 6.0;
        let i = h.floor() as i32;
        let f = h - i as f32;
        let p = value * (1.0 - saturation);
        let q = value * (1.0 - saturation * f);
        let t = value * (1.0 - saturation * (1.0 - f));
        
        let (r, g, b) = match i % 6 {
            0 => (value, t, p),
            1 => (q, value, p),
            2 => (p, value, t),
            3 => (p, q, value),
            4 => (t, p, value),
            _ => (value, p, q),
        };
        
        egui::Color32::from_rgb(
            (r * 255.0) as u8,
            (g * 255.0) as u8,
            (b * 255.0) as u8,
        )
    }

    pub fn update(&mut self, delta_time: f32) {
        self.time += delta_time;
        
        // Use iterator for better performance
        for pendulum in &mut self.pendulums {
            // Simple harmonic motion with damping
            let gravity = 9.8f32;
            // Use fast approximation for sin (small angles)
            let angle = pendulum.angle;
            let sin_angle = if angle.abs() < 0.5 {
                // Taylor series approximation for small angles: sin(x) ≈ x - x^3/6
                angle - angle * angle * angle / 6.0
            } else {
                angle.sin()
            };
            
            let acceleration = -gravity / pendulum.length * sin_angle;
            
            pendulum.angular_velocity += acceleration * delta_time;
            pendulum.angle += pendulum.angular_velocity * delta_time;
            
            // Add some damping (use multiplication instead of pow for performance)
            pendulum.angular_velocity *= 0.9999;
        }
    }
    
    pub fn reset(&mut self) {
        self.time = 0.0;
        for pendulum in &mut self.pendulums {
            pendulum.angle = std::f32::consts::FRAC_PI_4;
            pendulum.angular_velocity = 0.0;
        }
    }

    pub fn draw(&mut self, painter: &egui::Painter, rect: egui::Rect) {
        let pivot_y = rect.center().y - 200.0;
        let scale = rect.height() * 0.25;
        let rod_angle = 0.1f32;
        let rod_tan = rod_angle.tan();
        
        // Draw perspective rod at an angle for better 3D effect
        self.draw_angled_rod(painter, rect, pivot_y);
        
        // Draw each pendulum with proper perspective spacing
        for (i, pendulum) in self.pendulums.iter().enumerate().rev() {
            let depth_factor = self.depth_factors[i];
            let pivot_x = rect.center().x - self.total_width * 0.5 + self.spacing_offsets[i];
            
            // Apply angled rod perspective - pivot points follow the rod angle
            let rod_y_offset = (pivot_x - rect.center().x) * rod_tan;
            let depth_y_offset = i as f32 * 8.0;
            let adjusted_pivot_y = pivot_y + depth_y_offset + rod_y_offset;
            
            let bob_x = pivot_x + pendulum.angle.sin() * pendulum.length * scale * depth_factor;
            let bob_y = adjusted_pivot_y + pendulum.angle.cos() * pendulum.length * scale * depth_factor;
            
            // Draw string
            painter.line_segment(
                [egui::pos2(pivot_x, adjusted_pivot_y), egui::pos2(bob_x, bob_y)],
                egui::Stroke::new(2.5 * depth_factor, pendulum.color),
            );
            
            // Draw bob with perspective
            self.draw_bob_perspective(painter, bob_x, bob_y, pendulum.color, depth_factor);
            
            // Draw pivot point
            painter.circle_filled(
                egui::pos2(pivot_x, adjusted_pivot_y),
                4.0 * depth_factor,
                egui::Color32::from_rgb(200, 200, 220),
            );
            
            // Draw pendulum info
            painter.text(
                egui::pos2(pivot_x - 15.0, adjusted_pivot_y - 25.0),
                egui::Align2::LEFT_CENTER,
                format!("P{}", i + 1),
                egui::FontId::proportional(12.0 * depth_factor),
                pendulum.color,
            );
        }
        
        // Draw wave pattern visualization
        self.draw_wave_pattern(painter, rect);
    }

    fn draw_angled_rod(&self, painter: &egui::Painter, rect: egui::Rect, pivot_y: f32) {
        let rod_angle = 0.20f32;
        let rod_tan = -rod_angle.tan();
        
        let left_x = rect.center().x - self.total_width * 0.5 - 20.0;
        let right_x = rect.center().x + self.total_width * 0.5 + 20.0;
        
        let left_y = pivot_y - (left_x - rect.center().x) * rod_tan + 20.0;
        let right_y = pivot_y - (right_x - rect.center().x) * rod_tan + 20.0;
        let rod_thickness = 10.0;
        
        // Main rod body
        painter.line_segment(
            [egui::pos2(left_x, left_y), egui::pos2(right_x, right_y)],
            egui::Stroke::new(rod_thickness, egui::Color32::from_rgb(139, 69, 19)),
        );
        
        // Rod top highlight
        painter.line_segment(
            [egui::pos2(left_x, left_y - rod_thickness * 0.3), egui::pos2(right_x, right_y - rod_thickness * 0.3)],
            egui::Stroke::new(2.0, egui::Color32::from_rgb(200, 160, 120)),
        );
        
        // Rod bottom shadow
        painter.line_segment(
            [egui::pos2(left_x, left_y + rod_thickness * 0.3), egui::pos2(right_x, right_y + rod_thickness * 0.3)],
            egui::Stroke::new(1.5, egui::Color32::from_rgb(100, 50, 20)),
        );
    }

    fn draw_bob_perspective(&self, painter: &egui::Painter, bob_x: f32, bob_y: f32, color: egui::Color32, depth_factor: f32) {
        let bob_size = 16.0 * depth_factor;
        
        // Main bob
        painter.circle_filled(
            egui::pos2(bob_x, bob_y),
            bob_size,
            color,
        );
        
        // Bob highlight for 3D effect
        let highlight_size = bob_size * 0.4;
        painter.circle_filled(
            egui::pos2(bob_x - bob_size * 0.25, bob_y - bob_size * 0.25),
            highlight_size,
            color.gamma_multiply(1.6),
        );
    }

    fn draw_wave_pattern(&mut self, painter: &egui::Painter, rect: egui::Rect) {
        let wave_rect = egui::Rect::from_min_size(
            egui::pos2(rect.left() + 20.0, rect.bottom() - 180.0),
            egui::vec2(rect.width() - 40.0, 150.0),
        );
        self.wave_rect = wave_rect;
        
        // Draw wave container
        painter.rect_filled(
            wave_rect,
            4.0,
            egui::Color32::from_rgb(40, 40, 50),
        );
        
        painter.rect_stroke(
            wave_rect,
            4.0,
            egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 100, 120)),
        );
        
        let time_scale = 0.5;
        let amplitude = 60.0;
        let width = wave_rect.width() as usize;
        let step = 2;
        let num_points = (width + step - 1) / step;
        
        // Reuse wave points vector to avoid reallocations
        if self.wave_points[0].len() != num_points {
            for points in &mut self.wave_points {
                points.clear();
                points.reserve(num_points);
            }
        }
        
        // Draw wave for each pendulum
        for (i, pendulum) in self.pendulums.iter().enumerate() {
            let color = pendulum.color;
            let phase = (self.time * std::f32::consts::TAU / pendulum.period) % std::f32::consts::TAU;
            
            // Reuse the vector for this pendulum
            let points = &mut self.wave_points[i];
            points.clear();
            
            // Precompute constants
            let inv_width = 1.0 / wave_rect.width();
            let time_scale_tau = time_scale * std::f32::consts::TAU;
            let center_y = wave_rect.center().y;
            
            // Generate points with optimized loop
            for x_step in 0..num_points {
                let x = (x_step * step) as f32;
                let x_pos = wave_rect.left() + x;
                let time_offset = (x * inv_width) * time_scale_tau;
                let y_offset = (phase + time_offset).sin() * amplitude;
                let y_pos = center_y + y_offset;
                points.push(egui::pos2(x_pos, y_pos));
            }
            
            if points.len() > 1 {
                painter.add(egui::Shape::line(points.clone(), egui::Stroke::new(2.0, color)));
            }
            
            // Draw label
            painter.text(
                egui::pos2(wave_rect.left() - 25.0, wave_rect.top() + 10.0 + (i as f32) * 25.0),
                egui::Align2::RIGHT_CENTER,
                format!("P{}", i + 1),
                egui::FontId::proportional(12.0),
                color,
            );
        }
        
        // Draw grid lines
        self.draw_wave_grid(painter, wave_rect);
        
        // Draw title
        painter.text(
            egui::pos2(wave_rect.center().x, wave_rect.top() - 15.0),
            egui::Align2::CENTER_CENTER,
            "Wave Pattern Visualization",
            egui::FontId::proportional(14.0),
            egui::Color32::from_rgb(220, 220, 255),
        );
    }

    fn draw_wave_grid(&self, painter: &egui::Painter, wave_rect: egui::Rect) {
        let grid_color = egui::Color32::from_rgba_unmultiplied(100, 100, 120, 80);
        let center_y = wave_rect.center().y;
        let amplitude = 60.0;
        
        // Zero line
        painter.line_segment(
            [egui::pos2(wave_rect.left(), center_y), egui::pos2(wave_rect.right(), center_y)],
            egui::Stroke::new(1.0, grid_color),
        );
        
        // Top amplitude line
        painter.line_segment(
            [egui::pos2(wave_rect.left(), center_y - amplitude), egui::pos2(wave_rect.right(), center_y - amplitude)],
            egui::Stroke::new(1.0, grid_color),
        );
        
        // Bottom amplitude line
        painter.line_segment(
            [egui::pos2(wave_rect.left(), center_y + amplitude), egui::pos2(wave_rect.right(), center_y + amplitude)],
            egui::Stroke::new(1.0, grid_color),
        );
        
        // Vertical grid lines
        let width_quarter = wave_rect.width() / 4.0;
        for i in 0..5 {
            let x = wave_rect.left() + (i as f32) * width_quarter;
            painter.line_segment(
                [egui::pos2(x, wave_rect.top()), egui::pos2(x, wave_rect.bottom())],
                egui::Stroke::new(1.0, grid_color),
            );
        }
    }
}