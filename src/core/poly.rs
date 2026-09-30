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

macro_rules! impl_poly {
    ($ty:ident<$n:literal>, $first:ident $(, $coeff:ident)* $(,)?) => {
        impl $ty<$n> {
            /// Evaluates the polynomial's value at x, using Horners rule
            pub const fn value_at(&self, x: f64) -> f64 {
                let [$first $(, $coeff)*] = self.coeffs;

                let mut value = $first;

                impl_poly!(@value value, x; $($coeff),*);

                value
            }

            /// Evaluates the polynomial's value and derivative at x, using Horners rule
            pub fn value_and_derivative_at(&self, x: f64) -> (f64, f64) {
                let [$first $(, $coeff)*] = self.coeffs;

                let mut value = $first;
                let mut derivative = 0.0;

                impl_poly!(@vd value, derivative, x; $($coeff),*);

                (value, derivative)
            }
        }
    };

    // Nothing left to process.
    (@value $value:ident, $x:ident;) => {};

    // Process one coefficient.
    (@value $value:ident, $x:ident; $coeff:ident $(, $rest:ident)*) => {
        $value = $value * $x + $coeff;

        impl_poly!(@value $value, $x; $($rest),*);
    };

    // Nothing left to process.
    (@vd $value:ident, $derivative:ident, $x:ident;) => {};

    // Process one coefficient.
    (@vd $value:ident, $derivative:ident, $x:ident; $coeff:ident $(, $rest:ident)*) => {
        $derivative = $derivative * $x + $value;
        $value = $value * $x + $coeff;

        impl_poly!(@vd $value, $derivative, $x; $($rest),*);
    };
}

impl_poly!(Polynomial<2>, a, b);
impl_poly!(Polynomial<3>, a, b, c);
impl_poly!(Polynomial<4>, a, b, c, d);
impl_poly!(Polynomial<5>, a, b, c, d, e);
impl_poly!(Polynomial<6>, a, b, c, d, e, f);

pub fn solve_linear(a: f64, b: f64) -> ArrayVec<f64, 1> {
    let mut roots = ArrayVec::new();
    if a != 0.0 {
        roots.push(-b / a);
    }
    roots
}

pub fn solve_quadratic(a: f64, b: f64, c: f64) -> ArrayVec<f64, 2> {
    let mut roots = ArrayVec::new();
    let disc = b * b - 4.0 * a * c;
    if disc >= 0.0 {
        let d = -0.5 * (b + b.signum() * disc.sqrt());
        let x0 = c / d;
        let x1 = d / a;
        roots.push(x0.min(x1));
        roots.push(x0.max(x1));
    }
    roots
}

pub type Linear = Polynomial<2>;
impl Polynomial<2> {
    pub fn roots(&self) -> ArrayVec<f64, 1> {
        let [a, b] = self.coeffs;
        solve_linear(a, b)
    }
}

pub type Quadratic = Polynomial<3>;
impl Quadratic {
    pub fn derivative(&self) -> Linear {
        let [a, b, _] = self.coeffs;
        Polynomial::new([2.0 * a, b])
    }

    pub fn roots(&self) -> ArrayVec<f64, 2> {
        let [a, b, c] = self.coeffs;
        solve_quadratic(a, b, c)
    }

    pub fn roots_between(&self, x_min: f64, x_max: f64) -> ArrayVec<f64, 2> {
        let mut roots = self.roots();
        roots.retain(|r| *r >= x_min && *r <= x_max);
        roots
    }
}

pub type Cubic = Polynomial<4>;
impl Cubic {
    pub fn derivative(&self) -> Quadratic {
        let [a, b, c, _] = self.coeffs;
        Polynomial::new([3.0 * a, 2.0 * b, c])
    }

    pub fn roots_between(&self, x_min: f64, x_max: f64, tol: f64) -> ArrayVec<f64, 3> {
        let mut roots = ArrayVec::new();
        let mut start = (x_min, self.value_at(x_min));
        let derivative_roots = self.derivative().roots_between(x_min, x_max);
        for end in derivative_roots.into_iter().chain(std::iter::once(x_max)) {
            let current = (end, self.value_at(end));
            if let Some(root) = bisect(start, current, tol, |x| self.value_and_derivative_at(x)) {
                roots.push(root);
            }
            start = current;
        }
        roots
    }
}

pub type Quartic = Polynomial<5>;
impl Quartic {
    pub const fn derivative(&self) -> Cubic {
        let [a, b, c, d, _] = self.coeffs;
        Polynomial::new([4.0 * a, 3.0 * b, 2.0 * c, d])
    }
}

pub type Quintic = Polynomial<6>;
impl Quintic {
    pub const fn derivative(&self) -> Quartic {
        let [a, b, c, d, e, _] = self.coeffs;
        Polynomial::new([5.0 * a, 4.0 * b, 3.0 * c, 2.0 * d, e])
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
                (f64::NEG_INFINITY, -self.a),
                (-xx, self.eval(-xx)),
                (xx, self.eval(xx)),
                (f64::INFINITY, self.a),
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
    (mut x_min, y_min): (f64, f64),
    (mut x_max, y_max): (f64, f64),
    tol: f64,
    f: impl Fn(f64) -> (f64, f64),
) -> Option<f64> {
    const DELTA: f64 = 5.0;
    if y_min.is_sign_positive() == y_max.is_sign_positive() {
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
        return None;
    };

    loop {
        let (y, dy) = f(x);
        if y.is_sign_positive() == y_min.is_sign_positive() {
            x_min = x;
        } else {
            x_max = x;
        }

        let x_newton_raphson = x - y / dy;
        let x_next = if x_newton_raphson > x_min && x_newton_raphson < x_max {
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

        if (x_next - x).abs() < tol {
            return Some(x_next);
        }
        x = x_next;
    }
}
