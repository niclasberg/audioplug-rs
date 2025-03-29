use std::marker::PhantomData;

use crate::core::Interpolate;

pub trait TransitionState<T>: Sized {
    fn update(&mut self, dt: f64) -> Option<T>;
}

pub trait Transition<T> {
    type State: TransitionState<T>;

    fn create_state(&self, from: &T, to: &T) -> Self::State;
}

pub struct TweenState<T: Interpolate> {
    from: T,
    to: T,
    progress: f64,
    progress_per_second: f64,
    tween_fn: fn(progress: f64) -> f64,
}

impl<T: Interpolate> TransitionState<T> for TweenState<T> {
    fn update(&mut self, dt: f64) -> Option<T> {
        if self.progress >= 1.0 {
            None
        } else {
            self.progress = (self.progress + self.progress_per_second * dt).min(1.0);
            Some(self.from.lerp(&self.to, (self.tween_fn)(self.progress)))
        }
    }
}

pub struct Tween<T: Interpolate> {
    duration: f64,
    tween_fn: fn(progress: f64) -> f64,
    _phantom: PhantomData<T>,
}

impl<T: Interpolate> Tween<T> {
    pub fn linear(duration: f64) -> Self {
        Self {
            duration,
            tween_fn: |t| t,
            _phantom: PhantomData,
        }
    }
}

impl<T: Interpolate + Clone> Transition<T> for Tween<T> {
    type State = TweenState<T>;

    fn create_state(&self, from: &T, to: &T) -> Self::State {
        Self::State {
            from: from.clone(),
            to: to.clone(),
            progress: 0.0,
            progress_per_second: 1.0 / self.duration,
            tween_fn: self.tween_fn,
        }
    }
}
