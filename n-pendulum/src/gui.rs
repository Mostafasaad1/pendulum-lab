use eframe::egui;
use std::time::Instant;
use std::collections::VecDeque;

use crate::pendulum::{MAX_LINKS, HISTORY_SECONDS, HISTORY_SAMPLES, LinkParams};
use crate::solver::step_rk4;

pub struct NPendulumApp {
    pub n: usize,
    pub params: [LinkParams; MAX_LINKS],
    pub theta: [f32; MAX_LINKS],
    pub omega: [f32; MAX_LINKS],
    pub init_theta: [f32; MAX_LINKS],
    pub histories: [VecDeque<(f32,f32)>; MAX_LINKS],
    pub last_update: Option<Instant>,
    pub start_time: Instant,
    pub k1: Vec<f32>, k2: Vec<f32>, k3: Vec<f32>, k4: Vec<f32>,
    pub draw_points: Vec<egui::Pos2>,
}

impl Default for NPendulumApp {
    fn default() -> Self {
        let default_param = LinkParams { length: 1.0, mass: 1.0 };
        NPendulumApp {
            n: 3,
            params: [default_param; MAX_LINKS],
            init_theta: { let mut a = [0.0f32; MAX_LINKS]; a[0]=0.7; a[1]=0.4; a[2]=-0.3; a },
            theta: { let mut a = [0.0f32; MAX_LINKS]; a[0]=0.7; a[1]=0.4; a[2]=-0.3; a },
            omega: [0.0f32; MAX_LINKS],
            histories: [VecDeque::new(), VecDeque::new(), VecDeque::new(), VecDeque::new(), VecDeque::new(), VecDeque::new(), VecDeque::new()],
            last_update: None,
            start_time: Instant::now(),
            k1: vec![0.0f32; 2*MAX_LINKS], k2: vec![0.0f32; 2*MAX_LINKS], k3: vec![0.0f32; 2*MAX_LINKS], k4: vec![0.0f32; 2*MAX_LINKS],
            draw_points: Vec::with_capacity(MAX_LINKS),
        }
    }
}

impl NPendulumApp {
    pub fn reset_state(&mut self) {
        for i in 0..self.n { self.theta[i]=self.init_theta[i]; self.omega[i]=0.0; self.histories[i].clear(); }
        self.last_update=None; self.start_time=Instant::now();
    }

    pub fn step_rk4(&mut self, dt: f32) {
        let n = self.n;
        let mut lengths = [0.0f32; MAX_LINKS];
        let mut masses = [0.0f32; MAX_LINKS];
        for i in 0..n { lengths[i] = self.params[i].length; masses[i] = self.params[i].mass; }
        step_rk4(n, &lengths[..n], &masses[..n], &mut self.theta[..n], &mut self.omega[..n], dt, &mut self.k1, &mut self.k2, &mut self.k3, &mut self.k4);
    }

    pub fn push_histories(&mut self) {
        let t = self.start_time.elapsed().as_secs_f32();
        for i in 0..self.n {
            let h = &mut self.histories[i]; h.push_back((t, self.theta[i]));
            while let Some(&(old,_)) = h.front() { if t-old>HISTORY_SECONDS || h.len()>HISTORY_SAMPLES { h.pop_front(); } else { break; } }
        }
    }
}

impl eframe::App for NPendulumApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal_centered(|ui| { ui.heading("N-Pendulum Simulator"); });
        });

        // Controls window (compact)
        egui::Window::new("Controls")
            .anchor(egui::Align2::LEFT_TOP, egui::Vec2::new(8.0, 48.0))
            .resizable(true)
            .default_width(320.0)
            .min_width(260.0)
            .default_height(420.0)
            .show(ctx, |ui| {
                ui.heading("Controls");
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label("Links:"); let mut n = self.n as i32; ui.add(egui::Slider::new(&mut n, 1..=MAX_LINKS as i32).show_value(true)); self.n = n as usize;
                    if ui.button("Reset").clicked() { self.reset_state(); }
                });
                ui.add_space(4.0);
                let max_panel_h = 420.0f32.min(ui.available_height());
                egui::ScrollArea::vertical().max_height(max_panel_h).show(ui, |ui| {
                    ui.collapsing("Per-link settings", |ui| {
                        for i in 0..self.n {
                            ui.collapsing(format!("Link #{}", i+1), |ui| {
                                ui.horizontal(|ui| {
                                    ui.label("Length:");
                                    let mut len = self.params[i].length as f32;
                                    if ui.add(egui::DragValue::new(&mut len).speed(0.1)).changed() { self.params[i].length = len.max(0.01); }
                                    ui.label("m");
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Mass:");
                                    let mut mass = self.params[i].mass as f32;
                                    if ui.add(egui::DragValue::new(&mut mass).speed(0.1)).changed() { self.params[i].mass = mass.max(0.001); }
                                    ui.label("kg");
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Initial angle:");
                                    let mut deg = self.init_theta[i].to_degrees();
                                    if ui.add(egui::DragValue::new(&mut deg).speed(1.0)).changed() { self.init_theta[i] = deg.to_radians(); }
                                    ui.label("deg");
                                });
                            });
                        }
                    });
                });
            });

        // timing & integration
        let now = Instant::now(); let dt = if let Some(last)=self.last_update { now.duration_since(last).as_secs_f32() } else { 0.0 };
        self.last_update = Some(now);
        let dt = dt.min(0.05).max(0.0);
        if dt>0.0 { let steps = ((dt/0.005).ceil() as usize).max(1); let sub = dt/steps as f32; for _ in 0..steps { self.step_rk4(sub); self.push_histories(); } }

        egui::CentralPanel::default().show(ctx, |ui| {
            let available = ui.available_size();
            let canvas_fraction = 0.72f32; let min_canvas_h = 220.0f32;
            let canvas_height = (available.y * canvas_fraction).clamp(min_canvas_h, available.y - 80.0);
            let canvas = egui::Vec2::new(available.x, canvas_height);
            let (rect, _) = ui.allocate_exact_size(canvas, egui::Sense::hover());
            let painter = ui.painter_at(rect);
            let center = rect.center(); let mut x = center.x; let mut y = center.y - 20.0;
            self.draw_points.clear();
            for i in 0..self.n { let l = self.params[i].length*80.0; let ang = self.theta[i]; let nx = x + l*ang.sin(); let ny = y + l*ang.cos(); self.draw_points.push(egui::pos2(nx,ny)); x=nx; y=ny; }
            let mut prev = egui::pos2(center.x, center.y-20.0);
            for (i,p) in self.draw_points.iter().enumerate() { painter.line_segment([prev,*p], (2.0, egui::Color32::WHITE)); painter.circle_filled(*p,6.0, egui::Color32::from_rgb(200,100 + (i as u8*20),100)); prev = *p; }

            // plots grid
            let ideal_plot_w = 240.0f32; let gap = 8.0f32;
            let mut cols = (available.x / (ideal_plot_w + gap)).floor() as usize; if cols == 0 { cols = 1; }
            cols = cols.min(self.n.max(1)); let rows = (self.n + cols - 1) / cols;
            let plot_w = (available.x - gap * (cols as f32 + 1.0)) / cols as f32;
            let remaining_h = (available.y - canvas_height - 12.0).max(0.0);
            let mut plot_h = if rows > 0 { (remaining_h - gap * (rows as f32 + 1.0)) / rows as f32 } else { 120.0 };
            plot_h = plot_h.clamp(70.0, 160.0);
            let total_plots_h = rows as f32 * (plot_h + gap) + gap;

            if total_plots_h <= remaining_h {
                let top_filler = (remaining_h - total_plots_h).max(0.0);
                if top_filler > 0.0 { ui.add_space(top_filler); }
                for r in 0..rows {
                    ui.horizontal(|ui| {
                        ui.add_space(gap);
                        for c in 0..cols {
                            if c > 0 { ui.add_space(gap); }
                            let idx = r * cols + c;
                            ui.group(|ui| {
                                ui.vertical(|ui| {
                                    ui.horizontal(|ui| { if idx < self.n { ui.label(format!("Link #{}", idx+1)); } else { ui.label(""); } });
                                    let (rect, _) = ui.allocate_exact_size(egui::Vec2::new(plot_w, plot_h), egui::Sense::hover());
                                    if idx < self.n { draw_series_reuse(&ui.painter_at(rect), rect, &self.histories[idx], egui::Color32::from_rgb(120, 200 - (idx as u8 * 30), 150)); }
                                });
                            });
                        }
                    });
                    ui.add_space(gap);
                }
            } else {
                let max_h = (available.y - canvas_height - 12.0).max(80.0);
                egui::ScrollArea::vertical().max_height(max_h).show(ui, |ui| {
                    for r in 0..rows {
                        ui.horizontal(|ui| {
                            ui.add_space(gap);
                            for c in 0..cols {
                                if c > 0 { ui.add_space(gap); }
                                let idx = r * cols + c;
                                ui.group(|ui| {
                                    ui.vertical(|ui| {
                                        ui.horizontal(|ui| { if idx < self.n { ui.label(format!("Link #{}", idx+1)); } else { ui.label(""); } });
                                        let (rect, _) = ui.allocate_exact_size(egui::Vec2::new(plot_w, plot_h), egui::Sense::hover());
                                        if idx < self.n { draw_series_reuse(&ui.painter_at(rect), rect, &self.histories[idx], egui::Color32::from_rgb(120, 200 - (idx as u8 * 30), 150)); }
                                    });
                                });
                            }
                        });
                        ui.add_space(gap);
                    }
                });
            }
        });

        ctx.request_repaint();
    }
}

// leave draw helper in main module so both gui.rs and tests can call it easily
pub fn draw_series_reuse(painter: &egui::Painter, rect: egui::Rect, series: &VecDeque<(f32,f32)>, color: egui::Color32) {
    use egui::pos2;
    if series.len()<2 { painter.rect_stroke(rect, 0.0, (1.0, egui::Color32::from_gray(80))); return; }
    let t0 = series.front().unwrap().0; let t1 = series.back().unwrap().0; let dt = (t1-t0).max(1e-6);
    let mut minv=f32::INFINITY; let mut maxv=f32::NEG_INFINITY; for &(_,v) in series.iter() { minv=minv.min(v); maxv=maxv.max(v); }
    if (maxv-minv).abs()<1e-6 { maxv = minv+1.0; }
    let mut prev: Option<egui::Pos2> = None; for &(t,v) in series.iter() { let x = rect.left() + ((t-t0)/dt)*rect.width(); let y = rect.bottom() - ((v-minv)/(maxv-minv))*rect.height(); let p = pos2(x,y); if let Some(p0)=prev { painter.line_segment([p0,p], (1.5,color)); } prev = Some(p); }
    painter.rect_stroke(rect, 0.0, (1.0, egui::Color32::from_gray(80)));
}
