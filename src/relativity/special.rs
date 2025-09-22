pub const C: f64 = 299_792_458.0;

/// Lorentz factor γ = 1 / sqrt(1 - v^2/c^2)
pub fn lorentz_factor(v: f64) -> f64 {
    1.0 / (1.0 - (v * v) / (C * C)).sqrt()
}

/// Length contraction: L = L0 / γ
pub fn length_contraction(proper_length: f64, v: f64) -> f64 {
    let gamma = lorentz_factor(v);
    proper_length / gamma
}

