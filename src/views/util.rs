#[derive(Debug, Clone, Copy)]
pub struct ValueMapper {
    pub min: f64,
    pub max: f64,
    pub steps: usize,
}

impl ValueMapper {
    pub fn round(&self, value: f64) -> f64 {
        if self.steps == 0 {
            value
        } else {
            (value * self.steps as f64).round() / (self.steps as f64)
        }
    }

    pub fn normalize(&self, value: f64) -> f64 {
        ((value - self.min) / (self.max - self.min)).clamp(0.0, 1.0)
    }

    pub fn denormalize(&self, value: f64) -> f64 {
        self.min + (self.max - self.min) * value
    }
}
