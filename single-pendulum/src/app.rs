// src/app.rs

use std::collections::VecDeque;
use std::time::{Duration, Instant};

use eframe::{egui, Frame};
use egui::{Align, Context, Layout, Response, Sense};

use crate::physics::rk4_step;
use crate::plots::{draw_phase_plot, draw_time_series, PlotKind};
use crate::ui::draw_pendulum;

pub struct PendulumApp {
    // physics
    pub length: f32,
    pub mass: f32,
    pub drag: f32,
    pub gravity: f32,

    // simulation
    pub running: bool,
    start_instant: Instant,
    last_update: Instant,
    pub simulation_speed: f32,

    // state
    pub theta: f32,
    pub omega: f32,
    pub initial_theta: f32,

    // history: (time, theta_deg, omega_deg)
    history: VecDeque<(f32, f32, f32)>,
    history_capacity: usize,
    sample_accum: f32,
    sample_dt: f32,

    // UI & visualization
    pub plot_seconds: f32,
    pub show_timeline: bool,
    pub selected_plot: PlotKind,
    pub dark_mode: bool,
    pub auto_reset_history: bool,
    pub show_help: bool,

    // perf
    frame_count: u32,
    last_fps_update: Instant,
    pub current_fps: f32,

    // presets
    presets: Vec<Preset>,
    pub current_preset: usize,
    preset_to_apply: Option<usize>,
}

#[derive(Clone)]
pub struct Preset {
    pub name: String,
    pub length: f32,
    pub mass: f32,
    pub drag: f32,
    pub gravity: f32,
    pub initial_angle: f32,
}

impl Default for PendulumApp {
    fn default() -> Self {
        let presets = vec![
            Preset {
                name: "Simple".into(),
                length: 1.0,
                mass: 1.0,
                drag: 0.0,
                gravity: 9.81,
                initial_angle: 45.0,
            },
            Preset {
                name: "Damped".into(),
                length: 1.0,
                mass: 1.0,
                drag: 0.45,
                gravity: 9.81,
                initial_angle: 30.0,
            },
            Preset {
                name: "Long".into(),
                length: 2.0,
                mass: 0.6,
                drag: 0.08,
                gravity: 9.81,
                initial_angle: 60.0,
            },
        ];

        Self {
            length: 1.0,
            mass: 1.0,
            drag: 0.0,
            gravity: 9.81,
            running: false,
            start_instant: Instant::now(),
            last_update: Instant::now(),
            simulation_speed: 1.0,
            theta: 0.35,
            omega: 0.0,
            initial_theta: 0.35,
            history: VecDeque::with_capacity(4096),
            history_capacity: 4096,
            sample_accum: 0.0,
            sample_dt: 1.0 / 60.0,
            plot_seconds: 10.0,
            show_timeline: true,
            selected_plot: PlotKind::Angle,
            dark_mode: true,
            auto_reset_history: true,
            show_help: false,
            frame_count: 0,
            last_fps_update: Instant::now(),
            current_fps: 0.0,
            presets,
            current_preset: 0,
            preset_to_apply: None,
        }
    }
}

impl PendulumApp {
    fn push_history(&mut self, t: f32) {
        if self.auto_reset_history && self.history.len() == self.history_capacity {
            self.history.pop_front();
        }
        self.history
            .push_back((t, self.theta.to_degrees(), self.omega.to_degrees()));
    }

    pub fn apply_preset(&mut self, idx: usize) {
        if let Some(p) = self.presets.get(idx) {
            self.length = p.length;
            self.mass = p.mass;
            self.drag = p.drag;
            self.gravity = p.gravity;
            self.initial_theta = p.initial_angle.to_radians();
            self.theta = self.initial_theta;
            self.omega = 0.0;
            self.current_preset = idx;
            if self.auto_reset_history {
                self.history.clear();
            }
        }
    }

    pub fn calculate_energy(&self) -> (f32, f32, f32) {
        let potential = self.mass * self.gravity * self.length * (1.0 - self.theta.cos());
        let kinetic = 0.5 * self.mass * (self.length * self.omega).powi(2);
        (potential, kinetic, potential + kinetic)
    }

    fn clamp_parameters(&mut self) {
        self.length = self.length.clamp(0.1, 10.0);
        self.mass = self.mass.clamp(0.1, 10.0);
        self.drag = self.drag.clamp(0.0, 2.0);
        self.gravity = self.gravity.clamp(0.1, 20.0);
    }
}

impl eframe::App for PendulumApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        // FPS update
        let now = Instant::now();
        self.frame_count += 1;
        if (now - self.last_fps_update).as_secs_f32() >= 0.5 {
            let elapsed = (now - self.last_fps_update).as_secs_f32();
            self.current_fps = self.frame_count as f32 / elapsed;
            self.frame_count = 0;
            self.last_fps_update = now;
        }

        // apply pending preset
        if let Some(idx) = self.preset_to_apply.take() {
            self.apply_preset(idx);
        }

        // timestep
        let current_time = Instant::now();
        let mut dt = (current_time - self.last_update).as_secs_f32();
        if dt <= 0.0 {
            dt = 1.0 / 60.0;
        }
        self.last_update = current_time;
        dt = dt.min(0.05) * self.simulation_speed;

        self.clamp_parameters();

        // theme
        if self.dark_mode {
            ctx.set_visuals(egui::Visuals::dark());
        } else {
            ctx.set_visuals(egui::Visuals::light());
        }

        // LEFT: control panel (fixed width)
        let side_width = 320.0;
        egui::SidePanel::left("controls_panel")
            .resizable(false)
            .default_width(side_width)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.heading("Controls");
                    ui.add_space(8.0);

                    egui::Grid::new("controls_grid")
                        .num_columns(2)
                        .spacing([10.0, 8.0])
                        .show(ui, |ui| {
                            ui.label("Preset:");
                            egui::ComboBox::from_label("")
                                .selected_text(self.presets[self.current_preset].name.clone())
                                .show_ui(ui, |ui| {
                                    for (i, p) in self.presets.iter().enumerate() {
                                        if ui
                                            .selectable_value(
                                                &mut self.current_preset,
                                                i,
                                                p.name.clone(),
                                            )
                                            .clicked()
                                        {
                                            self.preset_to_apply = Some(i);
                                        }
                                    }
                                });
                            ui.end_row();

                            ui.label("Length (m):");
                            ui.add(egui::DragValue::new(&mut self.length).speed(0.1));
                            ui.end_row();
                            ui.label("Mass (kg):");
                            ui.add(egui::DragValue::new(&mut self.mass).speed(0.1));
                            ui.end_row();
                            ui.label("Drag:");
                            ui.add(egui::DragValue::new(&mut self.drag).speed(0.01));
                            ui.end_row();
                            ui.label("Gravity:");
                            ui.add(egui::DragValue::new(&mut self.gravity).speed(0.1));
                            ui.end_row();

                            ui.label("Init angle (°):");
                            let mut deg = self.initial_theta.to_degrees();
                            if ui.add(egui::DragValue::new(&mut deg).speed(1.0)).changed() {
                                self.initial_theta = deg.clamp(-179.0, 179.0).to_radians();
                            }
                            ui.end_row();

                            ui.label("Speed:");
                            ui.add(
                                egui::Slider::new(&mut self.simulation_speed, 0.1..=5.0)
                                    .fixed_decimals(1),
                            );
                            ui.end_row();
                            ui.label("Window:");
                            ui.add(egui::Slider::new(&mut self.plot_seconds, 1.0..=60.0));
                            ui.end_row();
                            ui.label("Auto-clear:");
                            ui.checkbox(&mut self.auto_reset_history, "");
                            ui.end_row();
                            ui.label("Timeline:");
                            ui.checkbox(&mut self.show_timeline, "");
                            ui.end_row();
                        });

                    ui.add_space(6.0);

                    ui.horizontal(|ui| {
                        let btn = if self.running { "⏸ Pause" } else { "▶ Start" };
                        if ui.add_sized([88.0, 30.0], egui::Button::new(btn)).clicked() {
                            self.running = !self.running;
                            self.last_update = Instant::now();
                            if self.running {
                                let t = (Instant::now() - self.start_instant).as_secs_f32();
                                self.push_history(t);
                            }
                        }
                        if ui
                            .add_sized([88.0, 30.0], egui::Button::new("🔄 Reset"))
                            .clicked()
                        {
                            self.theta = self.initial_theta;
                            self.omega = 0.0;
                            if self.auto_reset_history {
                                self.history.clear();
                            }
                            self.start_instant = Instant::now();
                        }
                        if ui
                            .add_sized([110.0, 30.0], egui::Button::new("Clear Data"))
                            .clicked()
                        {
                            self.history.clear();
                        }
                    });

                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);

                    ui.horizontal(|ui| {
                        ui.label("Plot:");
                        if ui
                            .selectable_label(self.selected_plot == PlotKind::Angle, "Angle")
                            .clicked()
                        {
                            self.selected_plot = PlotKind::Angle;
                        }
                        if ui
                            .selectable_label(self.selected_plot == PlotKind::Velocity, "Velocity")
                            .clicked()
                        {
                            self.selected_plot = PlotKind::Velocity;
                        }
                    });
                    ui.horizontal(|ui| {
                        if ui
                            .selectable_label(self.selected_plot == PlotKind::Energy, "Energy")
                            .clicked()
                        {
                            self.selected_plot = PlotKind::Energy;
                        }
                        if ui
                            .selectable_label(self.selected_plot == PlotKind::Phase, "Phase")
                            .clicked()
                        {
                            self.selected_plot = PlotKind::Phase;
                        }
                    });

                    ui.add_space(8.0);
                    ui.checkbox(&mut self.dark_mode, "Dark mode");
                    ui.add_space(6.0);
                    if ui.button("Help").clicked() {
                        self.show_help = !self.show_help;
                    }
                    if self.show_help {
                        ui.add_space(6.0);
                        ui.label("- Left: controls fixed width");
                        ui.label("- Center/Right: adaptive content shares same top alignment and height");
                    }
                });
            });

        // CENTER + RIGHT: adaptive layout with tighter alignment & responsive timeline
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                // header
                ui.horizontal(|ui| {
                    ui.heading("Pendulum Simulator — Adaptive Alignment");
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(format!(
                            "θ:{:.1}°  ω:{:.1}°/s  FPS:{:.1}",
                            self.theta.to_degrees(),
                            self.omega.to_degrees(),
                            self.current_fps
                        ));
                    });
                });
                ui.add_space(6.0);

                // compute available area and derive main content height so bottom isn't empty
                let avail = ui.available_size();

                // tighten header reserved height a bit (less empty space)
                let header_reserved = 48.0;

                // responsive timeline: base on available height so it shrinks when vertical space is tight
                let timeline_est = if self.show_timeline {
                    // use a small fraction of the full available height, but clamp to [64, 120]
                    ((avail.y * 0.12).round()).clamp(64.0, 120.0)
                } else {
                    0.0
                };

                let bottom_margin = 10.0;
                let main_height = (avail.y - header_reserved - timeline_est - bottom_margin).max(260.0);

                // responsive stacking threshold
                let total_width = ui.available_width();
                let should_stack = total_width < 980.0;

                if should_stack {
                    // stacked: pendulum on top, plots below (both use half main_height)
                    let each_h = (main_height / 2.0).max(200.0);

                    // pendulum block
                    let pend_size = egui::vec2(total_width, each_h);
                    let (resp_pend, painter_pend) = ui.allocate_painter(pend_size, Sense::hover());
                    draw_pendulum(self, &painter_pend, resp_pend.rect);
                    ui.add_space(6.0);

                    // main plot block
                    let plot_size = egui::vec2(total_width, each_h);
                    let (resp_plot, painter_plot) = ui.allocate_painter(plot_size, Sense::hover());
                    match self.selected_plot {
                        PlotKind::Angle => draw_time_series(
                            &painter_plot,
                            resp_plot.rect,
                            &self.history,
                            self.plot_seconds,
                            |(_, th, _)| *th,
                            Some((-180.0, 180.0)),
                            "Angle (°)",
                            egui::Color32::from_rgb(65, 105, 225),
                        ),
                        PlotKind::Velocity => draw_time_series(
                            &painter_plot,
                            resp_plot.rect,
                            &self.history,
                            self.plot_seconds,
                            |(_, _, w)| *w,
                            None,
                            "Angular Velocity (°/s)",
                            egui::Color32::from_rgb(220, 20, 60),
                        ),
                        PlotKind::Energy => draw_time_series(
                            &painter_plot,
                            resp_plot.rect,
                            &self.history,
                            self.plot_seconds,
                            |(_, th, _)| th.abs(),
                            None,
                            "Energy (proxy)",
                            egui::Color32::from_rgb(50, 200, 100),
                        ),
                        PlotKind::Phase => draw_phase_plot(&painter_plot, resp_plot.rect, &self.history),
                    }

                    // timeline (optional) - responsive height
                    if self.show_timeline {
                        ui.add_space(6.0);
                        let (resp_t, painter_t) = ui.allocate_painter(
                            egui::vec2(total_width, timeline_est),
                            Sense::hover(),
                        );
                        draw_time_series(
                            &painter_t,
                            resp_t.rect,
                            &self.history,
                            self.plot_seconds,
                            |(_, th, _)| *th,
                            Some((-90.0, 90.0)),
                            "Timeline",
                            egui::Color32::from_rgb(46, 139, 87),
                        );
                    }
                } else {
                    // wide layout: left = pendulum, right = plot (same top and same height)
                    let left_w = 0.52 * total_width; // pendulum width
                    let right_w = total_width - left_w - 12.0; // small spacing

                    // store responses so we can draw an alignment guide afterwards
                    let mut resp_pend_opt: Option<Response> = None;
                    let mut resp_plot_opt: Option<Response> = None;

                    ui.horizontal(|ui| {
                        // pendulum area
                        let pend_size = egui::vec2(left_w, main_height);
                        let (resp_pend, painter_pend) = ui.allocate_painter(pend_size, Sense::hover());
                        draw_pendulum(self, &painter_pend, resp_pend.rect);
                        resp_pend_opt = Some(resp_pend);

                        ui.add_space(10.0);

                        // plot area
                        let plot_size = egui::vec2(right_w, main_height);
                        let (resp_plot, painter_plot) = ui.allocate_painter(plot_size, Sense::hover());
                        match self.selected_plot {
                            PlotKind::Angle => draw_time_series(
                                &painter_plot,
                                resp_plot.rect,
                                &self.history,
                                self.plot_seconds,
                                |(_, th, _)| *th,
                                Some((-180.0, 180.0)),
                                "Angle (°)",
                                egui::Color32::from_rgb(65, 105, 225),
                            ),
                            PlotKind::Velocity => draw_time_series(
                                &painter_plot,
                                resp_plot.rect,
                                &self.history,
                                self.plot_seconds,
                                |(_, _, w)| *w,
                                None,
                                "Angular Velocity (°/s)",
                                egui::Color32::from_rgb(220, 20, 60),
                            ),
                            PlotKind::Energy => draw_time_series(
                                &painter_plot,
                                resp_plot.rect,
                                &self.history,
                                self.plot_seconds,
                                |(_, th, _)| th.abs(),
                                None,
                                "Energy (proxy)",
                                egui::Color32::from_rgb(50, 200, 100),
                            ),
                            PlotKind::Phase => draw_phase_plot(&painter_plot, resp_plot.rect, &self.history),
                        }
                        resp_plot_opt = Some(resp_plot);
                    });

                    // draw a subtle alignment guide connecting pendulum pivot to plot y-axis
                    if let (Some(resp_pend), Some(resp_plot)) = (resp_pend_opt, resp_plot_opt) {
                        let pivot = resp_pend.rect.center(); // we draw from pivot horizontally to the plot's left edge
                        let guide_x = resp_plot.rect.left();
                        let guide_top = resp_pend.rect.top();
                        let guide_bottom = resp_plot.rect.bottom();

                        // use central painter so the line appears above both widgets
                        let painter = ui.painter(); // central painter for this region

                        // faint vertical guide
                        painter.line_segment(
                            [egui::pos2(guide_x, guide_top), egui::pos2(guide_x, guide_bottom)],
                            egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(200, 200, 200, 50)),
                        );

                        // horizontal tick from pivot to guide (subtle)
                        painter.line_segment(
                            [egui::pos2(pivot.x, pivot.y), egui::pos2(guide_x, pivot.y)],
                            egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(200, 200, 200, 60)),
                        );

                        // small circle at pivot for clarity (very faint border)
                        painter.circle_filled(
                            pivot,
                            3.0,
                            egui::Color32::from_rgba_unmultiplied(200, 200, 200, 90),
                        );
                    }

                    // timeline below the plot area (aligned to right side width)
                    if self.show_timeline {
                        ui.add_space(6.0);
                        let timeline_width = right_w;
                        ui.horizontal(|ui| {
                            ui.add_space(left_w + 10.0); // shift to the right column position
                            let (resp_t, painter_t) = ui.allocate_painter(
                                egui::vec2(timeline_width, timeline_est),
                                Sense::hover(),
                            );
                            draw_time_series(
                                &painter_t,
                                resp_t.rect,
                                &self.history,
                                self.plot_seconds,
                                |(_, th, _)| *th,
                                Some((-90.0, 90.0)),
                                "Timeline",
                                egui::Color32::from_rgb(46, 139, 87),
                            );
                        });
                    }
                } // end adaptive branch
            }); // end central vertical
        });

        // PHYSICS integration
        let l = self.length.max(0.01);
        let m = self.mass.max(1e-6);
        let b = self.drag.max(0.0);
        let g = self.gravity.max(0.1);

        if self.running {
            let mut remaining = dt;
            let max_sub = 0.005_f32;
            while remaining > 0.0 {
                let step = remaining.min(max_sub);
                let (th, w) = rk4_step(self.theta, self.omega, step, l, m, b, g);
                self.theta = th;
                self.omega = w;
                remaining -= step;

                self.sample_accum += step;
                if self.sample_accum >= self.sample_dt {
                    let t = (Instant::now() - self.start_instant).as_secs_f32();
                    self.push_history(t);
                    self.sample_accum -= self.sample_dt;
                }
            }
            ctx.request_repaint_after(Duration::from_millis(16));
        }
    }
}