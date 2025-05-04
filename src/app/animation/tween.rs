use std::{
    any::Any,
    time::{Duration, Instant},
};

use crate::core::Interpolate;

use super::Animation;

type EasingFn = fn(f64) -> f64;

#[derive(Debug, Default, PartialEq, Eq)]
pub enum Easing {
    #[default]
    Linear,
    SineIn,
    SineOut,
    SineInOut,
    QuadIn,
    QuadOut,
    QuadInOut,
    CubicIn,
    CubicOut,
    CubicInOut,
    Custom(EasingFn),
}

impl Easing {
    fn into_fn(self) -> EasingFn {
        use std::f64::consts::PI;
        match self {
            Self::Linear => |x| x,
            Self::SineIn => |x| 1.0 - ((x * PI) / 2.0).cos(),
            Self::SineOut => |x| ((x * PI) / 2.0).sin(),
            Self::SineInOut => |x| -((PI * x).cos() - 1.0) / 2.0,
            Self::QuadIn => |x| x.powi(2),
            Self::QuadOut => |x| 1.0 - (1.0 - x).powi(2),
            Self::QuadInOut => |x| {
                if x < 0.5 {
                    2.0 * x * x
                } else {
                    1.0 - (-2.0 * x + 2.0).powi(2) / 2.0
                }
            },
            Self::CubicIn => |x| x.powi(3),
            Self::CubicOut => |x| 1.0 - (1.0 - x).powi(3),
            Self::CubicInOut => |x| {
                if x < 0.5 {
                    4.0 * x.powi(3)
                } else {
                    1.0 - (-2.0 * x + 2.0).powi(3) / 2.0
                }
            },
            Self::Custom(f) => f,
        }
    }
}

pub struct TweenOptions {
    pub duration: Duration,
    pub easing: Easing,
}

impl TweenOptions {
    pub const fn new() -> Self {
        Self {
            duration: Duration::from_secs(1),
            easing: Easing::Linear,
        }
    }
}

impl Default for TweenOptions {
    fn default() -> Self {
        Self::new()
    }
}

enum TweenState<T> {
    Running { start_time: Instant, from: T, to: T },
    Done,
}

pub struct TweenAnimation<T> {
    state: TweenState<T>,
    value: T,
    duration: f64,
    tween_fn: fn(progress: f64) -> f64,
}

impl<T> TweenAnimation<T> {
    pub fn new(value: T, options: TweenOptions) -> Self {
        Self {
            duration: options.duration.as_secs_f64(),
            tween_fn: options.easing.into_fn(),
            state: TweenState::Done,
            value,
        }
    }
}

impl<T> Animation for TweenAnimation<T>
where
    T: Any + Clone + Interpolate,
{
    type Value = T;

    fn value(&self) -> &T {
        &self.value
    }

    fn drive(&mut self, time: Instant) -> bool {
        match &mut self.state {
            TweenState::Running {
                start_time,
                from,
                to,
            } => {
                let seconds_passed = time.duration_since(*start_time).as_secs_f64();
                if seconds_passed >= self.duration {
                    self.value = to.clone();
                    self.state = TweenState::Done;
                } else {
                    self.value = from.lerp(&to, (self.tween_fn)(seconds_passed / self.duration));
                }
                true
            }
            TweenState::Done => false,
        }
    }

    fn set_target(&mut self, value: &T) -> bool {
        self.state = TweenState::Running {
            start_time: Instant::now(),
            from: self.value.clone(),
            to: value.clone(),
        };
        true
    }

    fn set_target_and_value(&mut self, value: &T) {
        self.state = TweenState::Done;
        self.value = value.clone();
    }
}
