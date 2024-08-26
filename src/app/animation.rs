use num::Float;

pub trait Animation {
    fn tick(&mut self, delta_t: f64) -> Option<f64>;
}

pub struct SpringAnimation {
    x: f64,
    xdot: f64,
    end: f64,
    mass: f64,
    stiffness: f64
}