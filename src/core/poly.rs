use arrayvec::ArrayVec;
use std::ops::Range;

pub struct Polynomial<const N: usize> {
    coeffs: [f64; N],
}

impl<const N: usize> Polynomial<N> {
    pub const fn new(coeffs: [f64; N]) -> Self {
        Self { coeffs }
    }

    pub const fn coeffs(&self) -> &[f64; N] {
        &self.coeffs
    }
}

pub type Linear = Polynomial<2>;

impl Polynomial<2> {
    pub const fn value_at(&self, x: f64) -> f64 {
        let [a, b] = self.coeffs;
        a * x + b
    }

    pub fn value_and_derivative_at(&self, x: f64) -> (f64, f64) {
        let [a, b] = self.coeffs;
        (a * x + b, a)
    }

    pub fn roots(&self) -> ArrayVec<f64, 1> {
        let [a, b] = self.coeffs;
        let mut roots = ArrayVec::new();
        if a != 0.0 {
            roots.push(-b / a);
        }
        roots
    }
}

pub type Quadratic = Polynomial<3>;

impl Polynomial<3> {
    pub const fn value_at(&self, x: f64) -> f64 {
        let [a, b, c] = self.coeffs;
        x * (a * x + b) + c
    }

    pub const fn value_and_derivative_at(&self, x: f64) -> (f64, f64) {
        let [a, b, c] = self.coeffs;
        let mut dy = a;
        let mut y = x * a + b;
        dy = y + x * dy;
        y = x * y + c;
        (y, dy)
    }

    pub fn derivative(&self) -> Linear {
        let [a, b, _] = self.coeffs;
        Polynomial::new([2.0 * a, b])
    }

    pub fn roots(&self) -> ArrayVec<f64, 2> {
        let mut roots = ArrayVec::new();
        self.visit_roots(|x| roots.push(x));
        roots
    }

    pub fn roots_between(&self, x_min: f64, x_max: f64) -> ArrayVec<f64, 2> {
        let mut roots = ArrayVec::new();
        self.visit_roots(|x| {
            if x >= x_min && x <= x_max {
                roots.push(x);
            }
        });
        roots
    }

    fn visit_roots(&self, mut f: impl FnMut(f64)) {
        let [a, b, c] = self.coeffs;
        let disc = b * b - 4.0 * a * c;
        if disc >= 0.0 {
            let d = -0.5 * (b + b.signum() * disc.sqrt());
            let x0 = c / d;
            let x1 = d / a;
            f(x0.min(x1));
            f(x0.max(x1));
        }
    }
}

pub type Cubic = Polynomial<4>;

impl Cubic {
    pub fn value_at(&self, x: f64) -> f64 {
        let [a, b, c, d] = self.coeffs;
        x * (x * (a * x + b) + c) + d
    }

    pub const fn value_and_derivative_at(&self, x: f64) -> (f64, f64) {
        let [a, b, c, d] = self.coeffs;
        let mut dy = a;
        let mut y = x * a + b;
        dy = y + x * dy;
        y = x * y + c;
        dy = y + x * dy;
        y = x * y + d;
        (y, dy)
    }

    pub fn derivative(&self) -> Quadratic {
        let [a, b, c, _] = self.coeffs;
        Polynomial::new([3.0 * a, 2.0 * b, c])
    }

    pub fn roots_between(&self, x_min: f64, x_max: f64, tol: f64) -> ArrayVec<f64, 3> {
        let mut start = (x_min, self.value_at(x_min).signum());
        let end = (x_max, self.value_at(x_max).signum());
        let derivative = self.derivative();

        let mut roots = ArrayVec::new();
        let mut do_bisect = |start, end| {
            if let Some(root) = bisect(start, end, tol, |x| self.value_and_derivative_at(x)) {
                roots.push(root);
            }
        };

        derivative.visit_roots(|x| {
            if x < x_max && x > x_min {
                let current = (x, self.value_at(x).signum());
                do_bisect(start, current);
                start = current;
            }
        });
        do_bisect(start, end);
        roots
    }
}

pub type Quartic = Polynomial<5>;

impl Quartic {
    pub const fn value_at(&self, x: f64) -> f64 {
        let [a, b, c, d, e] = self.coeffs;
        x * (x * (x * (a * x + b) + c) + d) + e
    }
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
    pub fn value_and_derivative(&self, x: f64) -> (f64, f64) {
        (x * (x * x + self.a) + self.b, 3.0 * x * x + self.a)
    }

    pub fn solve(&self, bounds: Range<f64>, tol: f64) -> ArrayVec<f64, 3> {
        let mut roots = ArrayVec::new();
        if self.a >= 0.0 {
            // Find root with Newton raphson
            let mut x = 0.0;
            loop {
                let (y, dy) = self.value_and_derivative(x);
                if y.abs() < tol {
                    break;
                }
                x -= y / dy;
            }
            roots.push(x);
        } else {
            // Two extreme values, given by x = ± sqrt(-a / 3)
            let xx = (-self.a / 3.0).sqrt();
            let interval_points = [
                (f64::NEG_INFINITY, (-self.a).signum()),
                (-xx, self.eval(-xx).signum()),
                (xx, self.eval(xx).signum()),
                (f64::INFINITY, self.a.signum()),
            ];

            for i in 0..interval_points.len() - 1 {
                let start = interval_points[i];
                let end = interval_points[i + 1];
                let root = bisect(start, end, tol, |x| self.value_and_derivative(x));
                if let Some(root) = root {
                    roots.push(root);
                }
            }
        }
        roots
    }
}

/// Given a function, f, that is monotonic in the interval [x_min, x_max), this function returns a
/// single root, if it exists.
fn bisect(
    (mut x_min, y_min_sign): (f64, f64),
    (mut x_max, y_max_sign): (f64, f64),
    tol: f64,
    f: impl Fn(f64) -> (f64, f64),
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
        let (y, dy) = f(x);
        if y.abs() < tol {
            break Some(x);
        }

        if y.signum() == y_min_sign {
            x_min = x;
        } else {
            x_max = x;
        }

        let x_newton_raphson = x - y / dy;
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
