use std::f32;

pub const MAX_LINKS: usize = 7;
pub const HISTORY_SECONDS: f32 = 60.0;
pub const HISTORY_SAMPLES: usize = 1024;

#[derive(Clone, Copy)]
pub struct LinkParams { pub length: f32, pub mass: f32 }

// Non-method implementations that operate on plain slices so callers can
// use local copies of parameters and avoid borrowing a larger struct while
// mutably borrowing temporary buffers.
pub fn accelerations_impl(n: usize, lengths: &[f32], masses: &[f32], thetas: &[f32], _omegas: &[f32], out: &mut [f32]) {
    let g = 9.81f32;
    for i in 0..n {
        let mut torque = -g / lengths[i] * thetas[i].sin();
        if i > 0 { let k = 5.0f32; torque += -k * (thetas[i] - thetas[i - 1]); }
        if i + 1 < n { let k = 5.0f32; torque += -k * (thetas[i] - thetas[i + 1]); }
        let im = masses[i] * lengths[i] * lengths[i];
        out[i] = if im.abs() < 1e-12 { 0.0 } else { torque / im };
    }
}

pub fn deriv_impl(n: usize, lengths: &[f32], masses: &[f32], y: &[f32], out: &mut [f32]) {
    for i in 0..n { out[2 * i] = y[2 * i + 1]; }
    let mut thetas = [0.0f32; MAX_LINKS];
    let mut omegas = [0.0f32; MAX_LINKS];
    let mut acc = [0.0f32; MAX_LINKS];
    for i in 0..n { thetas[i] = y[2 * i]; omegas[i] = y[2 * i + 1]; }
    accelerations_impl(n, lengths, masses, &thetas[..n], &omegas[..n], &mut acc[..n]);
    for i in 0..n { out[2 * i + 1] = acc[i]; }
}
