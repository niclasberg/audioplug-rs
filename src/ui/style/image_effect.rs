#[derive(Clone)]
pub enum ImageEffect {
    GaussianBlur { radius: f64 },
    Opacity { value: f64 },
}
