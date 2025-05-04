use std::{any::Any, time::Instant};

use crate::core::{SpringPhysics, SpringProperties};

use super::Animation;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct SpringOptions {
    pub mass: f64,
    pub stiffness: f64,
    pub damping: f64,
}

impl Default for SpringOptions {
    fn default() -> Self {
        Self {
            mass: 1.0,
            stiffness: 120.0,
            damping: 21.9,
        }
    }
}

enum SpringState<T> {
    Static,
    Moving {
        last_time: Instant,
        elapsed: f64,
        velocity: T,
        target: T,
    },
}

pub struct SpringAnimation<T> {
    position: T,
    state: SpringState<T>,
    properties: SpringProperties,
}

impl<T: SpringPhysics> SpringAnimation<T> {
    pub fn new(initial_value: T, options: SpringOptions) -> Self {
        Self {
            position: initial_value,
            state: SpringState::Static,
            properties: SpringProperties {
                mass: options.mass,
                stiffness: options.stiffness,
                damping: options.damping,
                position_epsilon: 1.0e-3,
                velocity_epsilon: 1.0e-3,
            },
        }
    }
}

impl<T> Animation for SpringAnimation<T>
where
    T: SpringPhysics + Clone + Any,
{
    type Value = T;

    fn value(&self) -> &T {
        &self.position
    }

    fn drive(&mut self, time: Instant) -> bool {
        const DELTA_T: f64 = 1.0e-3;
        match &mut self.state {
            SpringState::Static => false,
            SpringState::Moving {
                last_time,
                velocity,
                elapsed,
                target,
            } => {
                *elapsed += time.duration_since(*last_time).as_secs_f64();
                *last_time = time;

                let mut converged = false;
                while !converged && *elapsed >= DELTA_T {
                    converged = self.position.apply_spring_update(
                        velocity,
                        DELTA_T,
                        target,
                        &self.properties,
                    );
                    *elapsed -= DELTA_T;
                }

                if converged {
                    self.position = target.clone();
                    self.state = SpringState::Static;
                }

                true
            }
        }
    }

    fn set_target(&mut self, value: &T) -> bool {
        let target = value.clone();
        let new_state = match &self.state {
            SpringState::Static => SpringState::Moving {
                last_time: Instant::now(),
                velocity: T::ZERO,
                elapsed: 0.0,
                target,
            },
            SpringState::Moving {
                last_time,
                velocity,
                elapsed,
                ..
            } => SpringState::Moving {
                last_time: *last_time,
                velocity: velocity.clone(),
                elapsed: *elapsed,
                target,
            },
        };
        self.state = new_state;
        true
    }

    fn set_target_and_value(&mut self, value: &T) {
        self.state = SpringState::Static;
        self.position = value.clone();
    }
}
