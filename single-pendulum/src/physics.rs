// src/physics.rs

pub fn rk4_step(theta: f32, omega: f32, dt: f32, l: f32, m: f32, b: f32, g: f32) -> (f32, f32) {
    let f = |th: f32, w: f32| -> (f32, f32) {
        let dth = w;
        let dw = -(g / l) * th.sin() - (b / m) * w;
        (dth, dw)
    };
    let (k1t, k1w) = f(theta, omega);
    let (k2t, k2w) = f(theta + 0.5 * dt * k1t, omega + 0.5 * dt * k1w);
    let (k3t, k3w) = f(theta + 0.5 * dt * k2t, omega + 0.5 * dt * k2w);
    let (k4t, k4w) = f(theta + dt * k3t, omega + dt * k3w);
    let new_theta = theta + (dt / 6.0) * (k1t + 2.0 * k2t + 2.0 * k3t + k4t);
    let new_omega = omega + (dt / 6.0) * (k1w + 2.0 * k2w + 2.0 * k3w + k4w);
    (new_theta, new_omega)
}