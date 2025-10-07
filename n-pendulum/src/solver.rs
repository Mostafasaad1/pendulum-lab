use crate::pendulum::{deriv_impl};
use crate::pendulum::MAX_LINKS;

/// Perform one RK4 step on the state arrays. `theta` and `omega` are updated in-place.
pub fn step_rk4(n: usize, params_lengths: &[f32], params_masses: &[f32], theta: &mut [f32], omega: &mut [f32], dt: f32, k1: &mut [f32], k2: &mut [f32], k3: &mut [f32], k4: &mut [f32]) {
    // Build a small stacked state vector y of size 2*n, using local arrays for safety.
    let mut y_local = [0.0f32; 2 * MAX_LINKS];
    for i in 0..n { y_local[2*i] = theta[i]; y_local[2*i+1] = omega[i]; }

    // k1
    deriv_impl(n, params_lengths, params_masses, &y_local[..2*n], k1);

    // k2
    let mut tmp = [0.0f32; 2 * MAX_LINKS];
    for i in 0..2*n { tmp[i] = y_local[i] + 0.5*dt*k1[i]; }
    deriv_impl(n, params_lengths, params_masses, &tmp[..2*n], k2);

    // k3
    for i in 0..2*n { tmp[i] = y_local[i] + 0.5*dt*k2[i]; }
    deriv_impl(n, params_lengths, params_masses, &tmp[..2*n], k3);

    // k4
    for i in 0..2*n { tmp[i] = y_local[i] + dt*k3[i]; }
    deriv_impl(n, params_lengths, params_masses, &tmp[..2*n], k4);

    // advance
    for i in 0..n {
        theta[i] += (dt/6.0) * (k1[2*i] + 2.0*k2[2*i] + 2.0*k3[2*i] + k4[2*i]);
        omega[i] += (dt/6.0) * (k1[2*i+1] + 2.0*k2[2*i+1] + 2.0*k3[2*i+1] + k4[2*i+1]);
    }
}
