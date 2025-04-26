use std::ops::{Add, Mul};

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

pub trait SpringComponents: Sized {}

pub struct SpringComponentArray<const N: usize>(pub [f64; N]);

impl<const N: usize> SpringComponents for SpringComponentArray<N> {}

pub struct SpringComponentVec(pub Vec<f64>);

impl SpringComponents for SpringComponentVec {}

pub trait SpringPhysics {
    type Components: SpringComponents;
    fn distance_to(&self, other: &Self) -> Self::Components;
    fn apply_spring_update(&self, values: Self::Components) -> Self;
}

impl SpringPhysics for f64 {
    type Components = SpringComponentArray<1>;

    fn distance_to(&self, other: &Self) -> Self::Components {
        SpringComponentArray([*self - *other])
    }

    fn apply_spring_update(&self, values: Self::Components) -> Self {
        *self + values.0[0]
    }
}

impl SpringPhysics for f32 {
    type Components = SpringComponentArray<1>;

    fn distance_to(&self, other: &Self) -> Self::Components {
        SpringComponentArray([(*self - *other) as _])
    }

    fn apply_spring_update(&self, values: Self::Components) -> Self {
        (*self as f64 + values.0[0]) as _
    }
}
