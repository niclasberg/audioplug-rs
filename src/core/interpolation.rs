pub trait Interpolate {
    fn lerp(&self, other: &Self, scalar: f64) -> Self;
}

impl Interpolate for f64 {
    fn lerp(&self, other: &Self, scalar: f64) -> Self {
        (1.0 - scalar) * self + scalar * other
    }
}

impl Interpolate for f32 {
    fn lerp(&self, other: &Self, scalar: f64) -> Self {
        self + ((other - self) * scalar as f32)
    }
}

impl Interpolate for i64 {
    fn lerp(&self, other: &Self, scalar: f64) -> Self {
        self + ((other - self) as f64 * scalar).round() as i64
    }
}

impl Interpolate for i32 {
    fn lerp(&self, other: &Self, scalar: f64) -> Self {
        self + ((other - self) as f64 * scalar).round() as i32
    }
}
