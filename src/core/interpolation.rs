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

impl<T: Interpolate, const N: usize> Interpolate for [T; N] {
    fn lerp(&self, other: &Self, scalar: f64) -> Self {
        std::array::from_fn(|i| self[i].lerp(&other[i], scalar))
    }
}

pub struct SpringProperties {
    pub mass: f64,
    pub stiffness: f64,
    pub damping: f64,
    pub position_epsilon: f64,
    pub velocity_epsilon: f64,
}

pub trait SpringPhysics: Sized + Interpolate {
    const ZERO: Self;
    fn apply_spring_update(
        &mut self,
        velocity: &mut Self,
        delta_t: f64,
        target: &Self,
        properties: &SpringProperties,
    ) -> bool;
}

impl SpringPhysics for f64 {
    const ZERO: Self = 0.0;

    fn apply_spring_update(
        &mut self,
        velocity: &mut Self,
        delta_t: f64,
        target: &Self,
        properties: &SpringProperties,
    ) -> bool {
        // Integrate using the semi implicit Euler method
        let spring_force = properties.stiffness * (*target - *self);
        let damping_force = -properties.damping * *velocity;
        *velocity += delta_t * (spring_force + damping_force) / properties.mass;
        *self += delta_t * *velocity;
        velocity.abs() < properties.velocity_epsilon
            && (*self - target).abs() < properties.position_epsilon
    }
}

impl SpringPhysics for f32 {
    const ZERO: Self = 0.0;

    fn apply_spring_update(
        &mut self,
        velocity: &mut Self,
        delta_t: f64,
        target: &Self,
        properties: &SpringProperties,
    ) -> bool {
        let mut pos = *self as f64;
        let mut vel = *velocity as f64;
        let converged =
            f64::apply_spring_update(&mut pos, &mut vel, delta_t, &(*target as f64), properties);
        *self = pos as _;
        *velocity = vel as _;
        converged
    }
}
