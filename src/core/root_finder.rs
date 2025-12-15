use std::ops::Range;

#[derive(Debug)]
pub struct Solution<const MAX_ROOTS: usize> {
    pub root_count: usize,
    pub roots: [f64; MAX_ROOTS],
}

impl<const MAX_ROOTS: usize> Default for Solution<MAX_ROOTS> {
    fn default() -> Self {
        Self {
            root_count: 0,
            roots: [0.0; MAX_ROOTS],
        }
    }
}

impl<const MAX_ROOTS: usize> IntoIterator for Solution<MAX_ROOTS> {
    type Item = f64;
    type IntoIter = SolutionIter<MAX_ROOTS>;

    fn into_iter(self) -> Self::IntoIter {
        SolutionIter {
            solution: self,
            i: 0,
        }
    }
}

pub struct SolutionIter<const MAX_ROOTS: usize> {
    solution: Solution<MAX_ROOTS>,
    i: usize,
}

impl<const MAX_ROOTS: usize> Iterator for SolutionIter<MAX_ROOTS> {
    type Item = f64;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i >= self.solution.root_count {
            None
        } else {
            let i = self.i;
            self.i += 1;
            Some(self.solution.roots[i])
        }
    }
}

#[derive(Clone, Copy)]
struct IntervalPoint {
    x: f64,
    y_sign: f64,
}

/// Represents a quadratic polynomial a*x^2 + b*x + c
pub struct Quadratic {
    pub coeffs: [f64; 3],
}

impl Quadratic {
    pub fn solve(&self) -> Solution<2> {
        let [a, b, c] = self.coeffs;
        let disc = b * b - 4.0 * a * c;
        if disc < 0.0 {
            Solution::default()
        } else {
            let d = b + b.signum() * disc.sqrt();
            Solution {
                root_count: 2,
                roots: [-2.0 * c / d, -d / (2.0 * a)],
            }
        }
    }
}

pub struct Cubic {
    pub coeffs: [f64; 4],
}

/// Represents the polynomial x^3 + a * x + b
pub struct DepressedCubic {
    pub a: f64,
    pub b: f64,
}

impl DepressedCubic {
    pub fn new(a: f64, b: f64) -> Self {
        Self { a, b }
    }

    #[inline(always)]
    pub fn eval(&self, x: f64) -> f64 {
        x * (x * x + self.a) + self.b
    }

    #[inline(always)]
    pub fn eval_derivative(&self, x: f64) -> f64 {
        3.0 * x * x + self.a
    }

    pub fn solve(&self, tol: f64) -> Solution<3> {
        if self.a >= 0.0 {
            // Find root with Newton raphson
            let mut x = 0.0;
            loop {
                let y = self.eval(x);
                if y.abs() < tol {
                    break;
                }
                x -= y / self.eval_derivative(x);
            }

            Solution {
                root_count: 1,
                roots: [x, 0.0, 0.0],
            }
        } else {
            let mut solution = Solution::default();
            // Two extreme values, given by x = ± sqrt(-a / 3)
            let xx = (-self.a / 3.0).sqrt();
            let interval_points = [
                IntervalPoint {
                    x: f64::NEG_INFINITY,
                    y_sign: (-self.a).signum(),
                },
                IntervalPoint {
                    x: -xx,
                    y_sign: self.eval(-xx).signum(),
                },
                IntervalPoint {
                    x: xx,
                    y_sign: self.eval(xx).signum(),
                },
                IntervalPoint {
                    x: f64::INFINITY,
                    y_sign: self.a.signum(),
                },
            ];

            for i in 0..interval_points.len() - 1 {
                let start = interval_points[i];
                let end = interval_points[i + 1];
                if let Some(root) = solve_in_interval(
                    start.x,
                    end.x,
                    start.y_sign,
                    end.y_sign,
                    tol,
                    |x| self.eval(x),
                    |x| self.eval_derivative(x),
                ) {
                    solution.roots[solution.root_count] = root;
                    solution.root_count += 1;
                }
            }

            solution
        }
    }
}

/// Given a monotonic function, f, returns a single root within the interval [x_min, x_max), if found
fn solve_in_interval(
    mut x_min: f64,
    mut x_max: f64,
    y_min_sign: f64,
    y_max_sign: f64,
    tol: f64,
    f: impl Fn(f64) -> f64,
    f_deriv: impl Fn(f64) -> f64,
) -> Option<f64> {
    const DELTA: f64 = 5.0;

    if y_min_sign == y_max_sign {
        return None;
    }

    // We might have critical points at x_min and/or x_max. Initialize away from those points
    let mut x = if x_min.is_finite() && x_max.is_finite() {
        0.5 * (x_min + x_max)
    } else if x_min.is_finite() {
        x_min + DELTA
    } else if x_max.is_finite() {
        x_max - DELTA
    } else {
        0.0
    };

    loop {
        let y = f(x);
        if y.abs() < tol {
            break Some(x);
        }

        if y.signum() == y_min_sign {
            x_min = x;
        } else {
            x_max = x;
        }

        let x_newton_raphson = x - y / f_deriv(x);
        x = if x > x_min && x < x_max {
            x_newton_raphson
        } else if x_min.is_finite() && x_max.is_finite() {
            // Fall back to bisection
            0.5 * (x_min + x_max)
        } else if x_min.is_finite() {
            x_min + DELTA
        } else if x_max.is_finite() {
            x_max - DELTA
        } else {
            unreachable!()
        };
    }
}

/// Represents a quartic polynomial a*x^4 + b*x^3 + c*x^2 + d*x + e
pub struct Quartic {
    pub coeffs: [f64; 5],
}

impl Quartic {
    pub fn new(a: f64, b: f64, c: f64, d: f64, e: f64) -> Self {
        Self {
            coeffs: [a, b, c, d, e],
        }
    }

    pub const fn eval(&self, x: f64) -> f64 {
        let [a, b, c, d, e] = self.coeffs;
        x * (x * (x * (a * x + b) + c) + d) + e
    }
}
