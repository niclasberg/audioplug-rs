use crate::core::{Point, Rect, SpringPhysics, Vec2};

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub enum FillRule {
    #[default]
    NonZero,
    EvenOdd,
}

#[derive(Copy, Clone, Debug)]
pub enum PathElement {
    MoveTo(Point),
    LineTo(Point),
    QuadTo(Point, Point),
    CurveTo(Point, Point, Point),
    ClosePath,
}

#[derive(Copy, Clone, Debug)]
pub enum PathSegment {
    Line(Line),
    QuadBezier(QuadBezier),
    CubicBezier(CubicBezier),
}

impl PathSegment {
    /// Evaluate the position at `t`
    pub fn eval(&self, t: f64) -> Point {
        match self {
            PathSegment::Line(line) => line.eval(t),
            PathSegment::QuadBezier(quad_bezier) => quad_bezier.eval(t),
            PathSegment::CubicBezier(cubic_bezier) => cubic_bezier.eval(t),
        }
    }

    /// Split the path segment at `t` into two separate segments.
    pub fn split(&self, t: f64) -> (Self, Self) {
        match self {
            PathSegment::Line(line) => {
                let (left, right) = line.split(t);
                (Self::Line(left), Self::Line(right))
            }
            PathSegment::QuadBezier(quad_bezier) => {
                let (left, right) = quad_bezier.split(t);
                (Self::QuadBezier(left), Self::QuadBezier(right))
            }
            PathSegment::CubicBezier(cubic_bezier) => {
                let (left, right) = cubic_bezier.split(t);
                (Self::CubicBezier(left), Self::CubicBezier(right))
            }
        }
    }

    pub fn bounds(&self) -> Rect {
        match self {
            PathSegment::Line(line) => line.bounds(),
            PathSegment::QuadBezier(quad_bezier) => quad_bezier.bounds(),
            PathSegment::CubicBezier(cubic_bezier) => cubic_bezier.bounds(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Path {
    elements: Vec<PathElement>,
}

impl Path {
    pub const fn new() -> Self {
        Self {
            elements: Vec::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            elements: Vec::with_capacity(capacity),
        }
    }

    pub fn add_rect(mut self, rect: Rect) -> Self {
        self.elements.push(PathElement::MoveTo(rect.top_left()));
        self.elements.push(PathElement::LineTo(rect.top_right()));
        self.elements.push(PathElement::LineTo(rect.bottom_right()));
        self.elements.push(PathElement::LineTo(rect.bottom_left()));
        self.elements.push(PathElement::ClosePath);
        self
    }

    pub fn move_to(mut self, to: Point) -> Self {
        self.elements.push(PathElement::MoveTo(to));
        self
    }

    pub fn line_to(mut self, to: Point) -> Self {
        self.elements.push(PathElement::LineTo(to));
        self
    }

    pub fn quad_to(mut self, control_point: Point, to: Point) -> Self {
        self.elements.push(PathElement::QuadTo(control_point, to));
        self
    }

    pub fn cubic_to(mut self, control_point1: Point, control_point2: Point, to: Point) -> Self {
        self.elements
            .push(PathElement::CurveTo(control_point1, control_point2, to));
        self
    }

    pub fn close_path(mut self) -> Self {
        self.elements.push(PathElement::ClosePath);
        self
    }

    /// Flattens the path into path elements corresponding to lines
    ///
    /// Adapted from Kurbo and (https://raphlinus.github.io/graphics/curves/2019/12/23/flatten-quadbez.html)
    pub fn flatten(&self, tolerance: f64, mut f: impl FnMut(Line)) {
        let mut last_point = None;
        let mut first_point = Point::ZERO;
        let sqrt_tolerance = tolerance.sqrt();
        for &el in self.elements.iter() {
            match el {
                PathElement::MoveTo(point) => {
                    last_point = Some(point);
                    first_point = point;
                }
                PathElement::LineTo(point) => {
                    f(Line::new(last_point.unwrap_or(Point::ZERO), point));
                    last_point = Some(point);
                }
                PathElement::QuadTo(p1, p2) => {
                    if let Some(p0) = last_point {
                        QuadBezier { p0, p1, p2 }.flatten(sqrt_tolerance, &mut f);
                    }
                    last_point = Some(p2);
                }
                PathElement::CurveTo(point, point1, point2) => todo!(),
                PathElement::ClosePath => {
                    if let Some(last_point) = last_point
                        && last_point.distance_squared_to(&first_point) > tolerance.powi(2)
                    {
                        f(Line::new(last_point, first_point));
                    }
                    first_point = last_point.unwrap_or(Point::ZERO);
                    last_point = None;
                }
            }
        }
    }

    pub fn bounds(&self) -> Rect {
        todo!()
    }

    pub fn offset(mut self, delta: Vec2) -> Self {
        for element in self.elements.iter_mut() {
            match element {
                PathElement::MoveTo(point) => *point += delta,
                PathElement::LineTo(point) => *point += delta,
                PathElement::QuadTo(point, point1) => {
                    *point += delta;
                    *point1 += delta;
                }
                PathElement::CurveTo(point, point1, point2) => {
                    *point += delta;
                    *point1 += delta;
                    *point2 += delta;
                }
                PathElement::ClosePath => {}
            }
        }
        self
    }
}

impl Default for Path {
    fn default() -> Self {
        Self::new()
    }
}

impl<P: IntoIterator<Item = PathElement>> From<P> for Path {
    fn from(value: P) -> Self {
        Path {
            elements: value.into_iter().collect(),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Line {
    pub p0: Point,
    pub p1: Point,
}

impl Line {
    pub fn new(p0: Point, p1: Point) -> Self {
        Self { p0, p1 }
    }

    pub fn eval(&self, t: f64) -> Point {
        eval_line(self.p0, self.p1, t)
    }

    pub fn closest_point_t(&self, pos: Point) -> f64 {
        let p0 = self.p0.into_vector();
        let p1 = self.p1.into_vector();
        (pos.into_vector() - p0).dot(p1 - p0).clamp(0.0, 1.0)
    }

    pub fn split(&self, t: f64) -> (Self, Self) {
        let pos_split = eval_line(self.p0, self.p1, t);
        let line1 = Self {
            p0: self.p0,
            p1: pos_split,
        };
        let line2 = Self {
            p0: pos_split,
            p1: self.p1,
        };
        (line1, line2)
    }

    pub fn bounds(&self) -> Rect {
        Rect::from_points(self.p0, self.p1)
    }

    pub fn into_quad_bezier(self) -> QuadBezier {
        QuadBezier {
            p0: self.p0,
            p1: eval_line(self.p0, self.p1, 0.5),
            p2: self.p1,
        }
    }

    pub fn into_cubic_bezier(self) -> CubicBezier {
        CubicBezier {
            p0: self.p0,
            p1: eval_line(self.p0, self.p1, 1.0 / 3.0),
            p2: eval_line(self.p0, self.p1, 2.0 / 3.0),
            p3: self.p1,
        }
    }

    pub fn clip_with_rect(mut self, rect: Rect) -> Option<Self> {
        // Clip using Cohen Sutherland algorithm
        const INSIDE: u8 = 0b0000;
        const LEFT: u8 = 0b0001;
        const RIGHT: u8 = 0b0010;
        const BOTTOM: u8 = 0b0100;
        const TOP: u8 = 0b1000;

        fn compute_out_code(p: Point, rect: Rect) -> u8 {
            let mut out = INSIDE;
            if p.x < rect.left {
                out |= LEFT;
            } else if p.x > rect.right {
                out |= RIGHT;
            }
            if p.y < rect.top {
                out |= TOP;
            } else if p.y > rect.bottom {
                out |= BOTTOM;
            }
            out
        }

        let mut outcode0 = compute_out_code(self.p0, rect);
        let mut outcode1 = compute_out_code(self.p1, rect);

        loop {
            if (outcode0 | outcode1) == INSIDE {
                return Some(self);
            } else if (outcode0 & outcode1) != 0 {
                // Both points are in the same outside zone (for instance both to the left).
                // i.e. both points are outside
                return None;
            } else {
                todo!()
            }
        }
    }
}

#[inline(always)]
const fn eval_line(p0: Point, p1: Point, t: f64) -> Point {
    Point::new((1.0 - t) * p0.x + t * p1.x, (1.0 - t) * p0.y + t * p1.y)
}

#[inline(always)]
const fn eval_quad(p0: Point, p1: Point, p2: Point, t: f64) -> Point {
    let u = 1.0 - t;
    Point::new(
        u * u * p0.x + 2.0 * u * t * p1.x + t * t * p2.x,
        u * u * p0.y + 2.0 * u * t * p1.y + t * t * p2.y,
    )
}

#[inline(always)]
const fn eval_cubic(p0: Point, p1: Point, p2: Point, p3: Point, t: f64) -> Point {
    let u = 1.0 - t;
    Point::new(
        u * u * u * p0.x + 3.0 * t * u * u * p1.x + 3.0 * t * t * u * p2.x + t * t * t * p3.x,
        u * u * u * p0.y + 3.0 * t * u * u * p1.y + 3.0 * t * t * u * p2.y + t * t * t * p3.y,
    )
}

#[derive(Debug, Copy, Clone)]
pub struct QuadBezier {
    /// Start point
    pub p0: Point,
    /// Control point
    pub p1: Point,
    /// End point
    pub p2: Point,
}

impl QuadBezier {
    pub fn new(p0: Point, p1: Point, p2: Point) -> Self {
        Self { p0, p1, p2 }
    }

    pub fn eval(&self, t: f64) -> Point {
        eval_quad(self.p0, self.p1, self.p2, t)
    }

    pub fn split(&self, t: f64) -> (Self, Self) {
        let pos_split = eval_quad(self.p0, self.p1, self.p2, t);
        let quad1 = Self {
            p0: self.p0,
            p1: eval_line(self.p0, self.p1, t),
            p2: pos_split,
        };
        let quad2 = Self {
            p0: pos_split,
            p1: eval_line(self.p1, self.p2, t),
            p2: self.p2,
        };
        (quad1, quad2)
    }

    pub fn bounds(&self) -> Rect {
        let mut min = self.p0.min(&self.p2);
        let mut max = self.p0.max(&self.p2);

        // If p1 is within the bounding box spanned by p0 and p2, then
        // the whole curve is bounded by p0 and p1. Otherwise,
        // we need to find the extreme point of the curve.
        if self.p1.x < min.x || self.p1.x > max.x || self.p1.y < min.y || self.p2.y > max.y {
            let tx = ((self.p0.x - self.p1.x) / (self.p0.x - 2.0 * self.p1.x + self.p2.x))
                .clamp(0.0, 1.0);
            let sx = 1.0 - tx;
            let ty = ((self.p0.y - self.p1.y) / (self.p0.y - 2.0 * self.p1.y + self.p2.y))
                .clamp(0.0, 1.0);
            let sy = 1.0 - ty;
            let px = sx * sx * self.p0.x + 2.0 * sx * tx * self.p1.x + tx * tx * self.p2.x;
            let py = sy * sy * self.p0.y + 2.0 * sy * ty * self.p1.y + ty * ty * self.p2.y;
            min.x = min.x.min(px);
            min.y = min.y.min(py);
            max.x = max.x.max(px);
            max.y = max.y.max(py);
        }

        Rect {
            left: min.x,
            top: min.y,
            right: max.x,
            bottom: max.y,
        }
    }

    pub fn into_cubic_bezier(self) -> CubicBezier {
        CubicBezier {
            p0: self.p0,
            p1: eval_line(self.p0, self.p1, 1.0 / 3.0),
            p2: eval_line(self.p1, self.p2, 2.0 / 3.0),
            p3: self.p2,
        }
    }

    pub fn flatten(&self, sqrt_tolerance: f64, f: &mut impl FnMut(Line)) {}
}

#[inline(always)]
fn weighted_point_sum<const N: usize>(point_weights: [(f64, Point); N]) -> Point {
    point_weights
        .iter()
        .copied()
        .fold(Point::ZERO, |p_acc, (w, p)| {
            Point::new(p_acc.x + w * p.x, p_acc.y + w * p.y)
        })
}

/// An approximation to $\int (1 + 4x^2) ^ -0.25 dx$
fn approx_parabola_integral(x: f64) -> f64 {
    const D: f64 = 0.67;
    x / (1.0 - D + (D.powi(4) + 0.25 * x * x).sqrt().sqrt())
}

/// An approximation to the inverse parabola integral.
fn approx_parabola_inv_integral(x: f64) -> f64 {
    const B: f64 = 0.39;
    x * (1.0 - B + (B * B + 0.25 * x * x).sqrt())
}

#[derive(Debug, Copy, Clone)]
pub struct CubicBezier {
    /// Start point
    pub p0: Point,
    pub p1: Point,
    pub p2: Point,
    pub p3: Point,
}

impl CubicBezier {
    pub fn eval(&self, t: f64) -> Point {
        eval_cubic(self.p0, self.p1, self.p2, self.p3, t)
    }

    pub fn split(&self, t: f64) -> (Self, Self) {
        let split_pos = eval_cubic(self.p0, self.p1, self.p2, self.p3, t);
        let cubic1 = Self {
            p0: self.p0,
            p1: eval_line(self.p0, self.p1, t),
            p2: eval_quad(self.p0, self.p1, self.p2, t),
            p3: split_pos,
        };
        let cubic2 = Self {
            p0: split_pos,
            p1: eval_quad(self.p1, self.p2, self.p3, t),
            p2: eval_line(self.p2, self.p3, t),
            p3: self.p3,
        };
        (cubic1, cubic2)
    }

    pub fn bounds(&self) -> Rect {
        todo!()
    }
}

struct Stoker {}
