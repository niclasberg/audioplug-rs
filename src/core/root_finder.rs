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

/// Solve the polynomial x^3 + a * x + b = 0 on an interval
pub fn solve_depressed_cubic(a: f64, b: f64, bounds: Range<f64>, tol: f64) -> Solution<3> {
    let eval = move |x: f64| x * (x * x + a) + b;

    if a >= 0.0 {
        // Single root and the function is monotonic.
        let y_start = eval(bounds.start);
        let y_end = eval(bounds.end);

        if y_start.signum() == y_end.signum() {
            // No sign change <=> no root within the interval
            Solution::default()
        } else {
            // Find root with Newton raphson
            let mut x = bounds.start + (bounds.end - bounds.start);
            loop {
                let y = x * (x * x + a) + b;
                if y.abs() < tol {
                    break;
                }
                x -= y / (3.0 * x * x + a);
            }

            Solution {
                root_count: 1,
                roots: [x, 0.0, 0.0],
            }
        }
    } else {
        // Two extreme values, given by x = ± sqrt(-a / 3)
        todo!()
    }
}
