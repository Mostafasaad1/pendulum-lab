// src/plots.rs

use std::collections::VecDeque;

use eframe::egui::{Align2, Color32, FontId, Painter, Pos2, Rect, Stroke};

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum PlotKind {
    Angle,
    Velocity,
    Energy,
    Phase,
}

pub fn draw_time_series<TExtract>(
    painter: &Painter,
    rect: Rect,
    history: &VecDeque<(f32, f32, f32)>,
    seconds_window: f32,
    extract: TExtract,
    fixed_range: Option<(f32, f32)>,
    title: &str,
    color: Color32,
) where
    TExtract: Fn(&(f32, f32, f32)) -> f32,
{
    painter.rect_filled(rect, 6.0, Color32::from_gray(22));
    painter.rect_stroke(rect, 6.0, Stroke::new(1.0, Color32::from_gray(90)));

    if history.is_empty() {
        if !title.is_empty() {
            painter.text(
                Pos2::new(rect.center().x, rect.center().y),
                Align2::CENTER_CENTER,
                "no data",
                FontId::proportional(14.0),
                Color32::from_gray(160),
            );
        }
        return;
    }

    let last_t = history.back().unwrap().0;
    let min_t = last_t - seconds_window;

    let mut pts: Vec<(f32, f32)> = Vec::with_capacity(history.len());
    let mut y_min = f32::INFINITY;
    let mut y_max = f32::NEG_INFINITY;
    for e in history.iter() {
        if e.0 < min_t {
            continue;
        }
        let v = extract(e);
        if !v.is_finite() {
            continue;
        }
        pts.push((e.0, v));
        y_min = y_min.min(v);
        y_max = y_max.max(v);
    }

    if pts.len() < 2 {
        if !title.is_empty() {
            painter.text(
                Pos2::new(rect.center().x, rect.center().y),
                Align2::CENTER_CENTER,
                "not enough samples",
                FontId::proportional(13.0),
                Color32::from_gray(150),
            );
        }
        return;
    }

    let (y_min, y_max) = if let Some(r) = fixed_range {
        r
    } else {
        let span = (y_max - y_min).abs().max(1e-3);
        (y_min - 0.12 * span, y_max + 0.12 * span)
    };

    // slightly tighter title spacing (4 px)
    if !title.is_empty() {
        painter.text(
            Pos2::new(rect.left() + 6.0, rect.top() + 6.0),
            Align2::LEFT_TOP,
            title,
            FontId::proportional(13.5),
            Color32::WHITE,
        );
    }

    let width = rect.width();
    let height = rect.height();
    let x_of = |t: f32| rect.left() + ((t - min_t) / seconds_window).clamp(0.0, 1.0) * width;
    let y_of = |v: f32| {
        if (y_max - y_min).abs() < 1e-6 {
            rect.center().y
        } else {
            rect.bottom() - ((v - y_min) / (y_max - y_min)).clamp(0.0, 1.0) * height
        }
    };

    let mid_y = y_of((y_min + y_max) * 0.5);
    painter.line_segment(
        [Pos2::new(rect.left(), mid_y), Pos2::new(rect.right(), mid_y)],
        Stroke::new(1.0, Color32::from_gray(85)),
    );

    let stroke = Stroke::new((2.0 + width / 420.0).min(4.0), color);
    let mut prev: Option<Pos2> = None;
    for (t, y) in &pts {
        let p = Pos2::new(x_of(*t), y_of(*y));
        if let Some(p0) = prev {
            painter.line_segment([p0, p], stroke);
        }
        prev = Some(p);
    }

    if let Some((t_last, y_last)) = pts.last() {
        painter.circle_filled(Pos2::new(x_of(*t_last), y_of(*y_last)), 3.0, color);
    }

    if !title.is_empty() {
        painter.text(
            Pos2::new(rect.left() + 6.0, rect.top() + 24.0),
            Align2::LEFT_TOP,
            format!("{:.1}", y_max),
            FontId::monospace(11.0),
            Color32::from_gray(200),
        );
        painter.text(
            Pos2::new(rect.left() + 6.0, rect.bottom() - 18.0),
            Align2::LEFT_TOP,
            format!("{:.1}", y_min),
            FontId::monospace(11.0),
            Color32::from_gray(200),
        );
        painter.text(
            Pos2::new(rect.right() - 60.0, rect.bottom() - 18.0),
            Align2::LEFT_TOP,
            format!("-{:.0}s", seconds_window),
            FontId::monospace(11.0),
            Color32::from_gray(170),
        );
    }
}

pub fn draw_phase_plot(painter: &Painter, rect: Rect, history: &VecDeque<(f32, f32, f32)>) {
    painter.rect_filled(rect, 6.0, Color32::from_gray(22));
    painter.rect_stroke(rect, 6.0, Stroke::new(1.0, Color32::from_gray(90)));
    painter.text(
        Pos2::new(rect.left() + 6.0, rect.top() + 6.0),
        Align2::LEFT_TOP,
        "Phase (θ vs ω)",
        FontId::proportional(13.5),
        Color32::WHITE,
    );

    if history.is_empty() {
        painter.text(
            Pos2::new(rect.center().x, rect.center().y),
            Align2::CENTER_CENTER,
            "no data",
            FontId::proportional(14.0),
            Color32::from_gray(160),
        );
        return;
    }

    let mut pts = Vec::new();
    let mut th_min = f32::INFINITY;
    let mut th_max = f32::NEG_INFINITY;
    let mut w_min = f32::INFINITY;
    let mut w_max = f32::NEG_INFINITY;
    for e in history.iter() {
        let th = e.1;
        let w = e.2;
        if th.is_finite() && w.is_finite() {
            pts.push((th, w));
            th_min = th_min.min(th);
            th_max = th_max.max(th);
            w_min = w_min.min(w);
            w_max = w_max.max(w);
        }
    }
    if pts.len() < 2 {
        return;
    }

    let th_span = (th_max - th_min).abs().max(1e-3);
    let w_span = (w_max - w_min).abs().max(1e-3);
    let th_min = th_min - 0.12 * th_span;
    let th_max = th_max + 0.12 * th_span;
    let w_min = w_min - 0.12 * w_span;
    let w_max = w_max + 0.12 * w_span;

    let width = rect.width();
    let height = rect.height();
    let x_of = |th: f32| rect.left() + ((th - th_min) / (th_max - th_min)).clamp(0.0, 1.0) * width;
    let y_of = |w: f32| {
        rect.bottom() - ((w - w_min) / (w_max - w_min)).clamp(0.0, 1.0) * height
    };

    painter.line_segment(
        [
            Pos2::new(rect.left(), y_of(0.0)),
            Pos2::new(rect.right(), y_of(0.0)),
        ],
        Stroke::new(1.0, Color32::from_gray(85)),
    );
    painter.line_segment(
        [
            Pos2::new(x_of(0.0), rect.top()),
            Pos2::new(x_of(0.0), rect.bottom()),
        ],
        Stroke::new(1.0, Color32::from_gray(85)),
    );

    let stroke = Stroke::new(1.6, Color32::from_rgb(200, 100, 255));
    let mut prev: Option<Pos2> = None;
    for (th, w) in pts.iter() {
        let p = Pos2::new(x_of(*th), y_of(*w));
        if let Some(p0) = prev {
            painter.line_segment([p0, p], stroke);
        }
        prev = Some(p);
    }
    if let Some((th, w)) = pts.last() {
        painter.circle_filled(Pos2::new(x_of(*th), y_of(*w)), 3.0, Color32::from_rgb(255, 255, 120));
    }
}