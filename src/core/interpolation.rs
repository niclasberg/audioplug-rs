use crate::core::Zero;

pub trait Lerp {
    fn lerp(&self, other: &Self, scalar: f64) -> Self;
}

impl Lerp for f64 {
    fn lerp(&self, other: &Self, scalar: f64) -> Self {
        (1.0 - scalar) * self + scalar * other
    }
}

impl Lerp for f32 {
    fn lerp(&self, other: &Self, scalar: f64) -> Self {
        self + ((other - self) * scalar as f32)
    }
}

impl Lerp for i64 {
    fn lerp(&self, other: &Self, scalar: f64) -> Self {
        self + ((other - self) as f64 * scalar).round() as i64
    }
}

impl Lerp for i32 {
    fn lerp(&self, other: &Self, scalar: f64) -> Self {
        self + ((other - self) as f64 * scalar).round() as i32
    }
}

impl<T: Lerp, const N: usize> Lerp for [T; N] {
    fn lerp(&self, other: &Self, scalar: f64) -> Self {
        std::array::from_fn(|i| self[i].lerp(&other[i], scalar))
    }
}

pub trait Interpolatable: Sized {
    fn interpolator(&self, other: &Self) -> impl Fn(f64) -> Self + 'static;
}

impl<T: Lerp + Clone + 'static> Interpolatable for T {
    fn interpolator(&self, other: &Self) -> impl Fn(f64) -> Self + 'static {
        let start = self.clone();
        let end = other.clone();
        move |scalar| start.lerp(&end, scalar)
    }
}

pub struct SpringProperties {
    pub mass: f64,
    pub stiffness: f64,
    pub damping: f64,
    pub position_epsilon: f64,
    pub velocity_epsilon: f64,
}

pub trait SpringPhysics: Sized + Lerp + Zero {
    fn distance_squared_to(&self, other: &Self) -> f64;
    fn apply_spring_update(
        &mut self,
        velocity: &mut Self,
        delta_t: f64,
        target: &Self,
        properties: &SpringProperties,
    );
}

impl SpringPhysics for f64 {
    fn distance_squared_to(&self, other: &Self) -> f64 {
        (*self - *other).powi(2)
    }

    fn apply_spring_update(
        &mut self,
        velocity: &mut Self,
        delta_t: f64,
        target: &Self,
        properties: &SpringProperties,
    ) {
        // Integrate using the semi implicit Euler method
        let spring_force = properties.stiffness * (*target - *self);
        let damping_force = -properties.damping * *velocity;
        *velocity += delta_t * (spring_force + damping_force) / properties.mass;
        *self += delta_t * *velocity;
    }
}

impl SpringPhysics for f32 {
    fn distance_squared_to(&self, other: &Self) -> f64 {
        (*self - *other).powi(2) as f64
    }

    fn apply_spring_update(
        &mut self,
        velocity: &mut Self,
        delta_t: f64,
        target: &Self,
        properties: &SpringProperties,
    ) {
        let mut pos = *self as f64;
        let mut vel = *velocity as f64;
        f64::apply_spring_update(&mut pos, &mut vel, delta_t, &(*target as f64), properties);
        *self = pos as _;
        *velocity = vel as _;
    }
}
