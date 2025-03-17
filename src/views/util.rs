pub fn normalize_value(min: f64, max: f64, value: f64) -> f64 {
	((value - min) / (max - min)).clamp(0.0, 1.0)
}

pub fn denormalize_value(min: f64, max: f64, value: f64) -> f64 {
    min + (max - min) * value
}

pub fn round_to_steps(steps: usize, value: f64) -> f64 {
    if steps == 0 {
        value
    } else {
        (value * steps as f64).round() / (steps as f64)
    }
}