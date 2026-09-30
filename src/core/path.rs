use crate::core::{
    Point, Rect, SpringPhysics, Transform, Vec2, Zero,
    poly::{DepressedCubic, Quartic},
};

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub enum FillRule {
    #[default]
    NonZero,
    EvenOdd,
}

#[derive(Debug, Copy, Clone)]
pub enum PathSegment {
    Line(Line),
    Quad(QuadBezier),
    Cubic(CubicBezier),
}

impl PathSegment {
    pub fn bounds(&self) -> Rect {
        match self {
            PathSegment::Line(line) => line.bounds(),
            PathSegment::Quad(quad_bezier) => quad_bezier.bounds(),
            PathSegment::Cubic(cubic_bezier) => cubic_bezier.bounds(),
        }
    }
}

impl From<Line> for PathSegment {
    fn from(value: Line) -> Self {
        Self::Line(value)
    }
}

impl From<QuadBezier> for PathSegment {
    fn from(value: QuadBezier) -> Self {
        Self::Quad(value)
    }
}

impl From<CubicBezier> for PathSegment {
    fn from(value: CubicBezier) -> Self {
        Self::Cubic(value)
    }
}

pub struct PathBuilder<'a> {
    segments: &'a mut Vec<PathSegment>,
    last_point: Option<Point>,
    first_point: Point,
}

impl<'a> PathBuilder<'a> {
    pub const fn new(segments: &'a mut Vec<PathSegment>) -> Self {
        Self {
            segments,
            last_point: None,
            first_point: Point::ZERO,
        }
    }

    pub fn move_to(&mut self, to: Point) -> &mut Self {
        self.last_point = Some(to);
        self.first_point = to;
        self
    }

    pub fn line_to(&mut self, to: Point) -> &mut Self {
        self.segments
            .push(Line::new(self.last_point.unwrap_or(Point::ZERO), to).into());
        self.last_point.replace(to);
        self
    }

    pub fn quad_to(&mut self, control_point: Point, to: Point) -> &mut Self {
        self.segments.push(
            QuadBezier::new(self.last_point.unwrap_or(Point::ZERO), control_point, to).into(),
        );
        self.last_point.replace(to);
        self
    }

    pub fn cubic_to(
        &mut self,
        control_point1: Point,
        control_point2: Point,
        to: Point,
    ) -> &mut Self {
        self.segments.push(
            CubicBezier::new(
                self.last_point.unwrap_or(Point::ZERO),
                control_point1,
                control_point2,
                to,
            )
            .into(),
        );
        self.last_point.replace(to);
        self
    }

    pub fn close_path(&mut self) -> &mut Self {
        if let Some(last_point) = self.last_point
            && last_point.distance_squared_to(&self.first_point) > 1.0e-4
        {
            self.segments
                .push(Line::new(last_point, self.first_point).into());
        }
        self.first_point = self.last_point.unwrap_or(Point::ZERO);
        self.last_point = None;
        self
    }
}

pub struct IntersectionPoint {
    pub point: Point,
    /// Parameter of intersection for the first path segment
    pub t1: f32,
    /// Parameter of intersection for the second path segment
    pub t2: f32,
}

#[derive(Debug, Copy, Clone)]
pub struct Line {
    pub p0: Point,
    pub p1: Point,
}

impl Line {
    pub fn new(p0: impl Into<Point>, p1: impl Into<Point>) -> Self {
        Self {
            p0: p0.into(),
            p1: p1.into(),
        }
    }

    pub fn eval(&self, t: f32) -> Point {
        eval_line(self.p0, self.p1, t)
    }

    pub fn closest_point_t(&self, pos: Point) -> f32 {
        (pos - self.p0).dot(self.p1 - self.p0).clamp(0.0, 1.0)
    }

    pub fn closest_point(&self, pos: Point) -> Point {
        self.eval(self.closest_point_t(pos))
    }

    pub fn distance_squared(&self, pos: Point) -> f32 {
        (self.closest_point(pos) - pos).length_squared()
    }

    pub fn distance(&self, pos: Point) -> f32 {
        (self.closest_point(pos) - pos).length()
    }

    pub fn split(&self, t: f32) -> (Self, Self) {
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

    pub fn intersect_line(&self, other: &Self) -> Option<IntersectionPoint> {
        // Intersect if (1-t1) * p10 + p11 = (1-t2) * p20 + p21
        // Use: https://www.sciencedirect.com/science/article/abs/pii/S0010448500000506
        None
    }
}

#[inline(always)]
const fn eval_line(p0: Point, p1: Point, t: f32) -> Point {
    Point {
        x: (1.0 - t) * p0.x + t * p1.x,
        y: (1.0 - t) * p0.y + t * p1.y,
    }
}

#[inline(always)]
const fn eval_quad(p0: Point, p1: Point, p2: Point, t: f32) -> Point {
    let u = 1.0 - t;
    Point {
        x: u * u * p0.x + 2.0 * u * t * p1.x + t * t * p2.x,
        y: u * u * p0.y + 2.0 * u * t * p1.y + t * t * p2.y,
    }
}

#[inline(always)]
const fn eval_cubic(p0: Point, p1: Point, p2: Point, p3: Point, t: f32) -> Point {
    let u = 1.0 - t;
    Point {
        x: u * u * u * p0.x + 3.0 * t * u * u * p1.x + 3.0 * t * t * u * p2.x + t * t * t * p3.x,
        y: u * u * u * p0.y + 3.0 * t * u * u * p1.y + 3.0 * t * t * u * p2.y + t * t * t * p3.y,
    }
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
    pub fn new(p0: impl Into<Point>, p1: impl Into<Point>, p2: impl Into<Point>) -> Self {
        Self {
            p0: p0.into(),
            p1: p1.into(),
            p2: p2.into(),
        }
    }

    #[inline(always)]
    fn points_as_vec2s(self) -> (Vec2, Vec2, Vec2) {
        (
            self.p0.into_vec2(),
            self.p1.into_vec2(),
            self.p2.into_vec2(),
        )
    }

    pub fn eval(&self, t: f32) -> Point {
        eval_quad(self.p0, self.p1, self.p2, t)
    }

    /// Finds the `t` value (in range 0 to 1), such that the
    /// distance to `pos` is minimal.
    pub fn closest_point_t(&self, pos: Point) -> f32 {
        // Distance squared is d(t) = |p(t) - pos|^2 = dot(p(t)-pos, p(t)-pos)
        // with derivative: d' = 2*dot(p(t)-pos, p'(t))
        // Now, p(t) = at^2 + bt + c, and p'(t) = 2at + b, where
        //   a = p0 - 2*p1 + p2
        //   b = -2*p0 + 2*p1
        //   c = p0
        // this gives:
        // 0 = dot(a, 2a) t^3 + (dot(a, b) + dot(b, 2a))t^2 + (dot(c-pos, 2a) + dot(b, b)) t + dot(c-pos, b)
        //   = 2|a|^2 t^3 + 3dot(a, b) t^2 + (2dot(c - pos, a) + |b|^2) + dot(c-pos, b)
        let dp1 = self.p1 - self.p0;

        0.0
    }

    pub fn split(&self, t: f32) -> (Self, Self) {
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
        let mut bounds = Rect::from_points(self.p0, self.p2);

        // If p1 is within the bounding box spanned by p0 and p2, then
        // the whole curve is bounded by p0 and p2. Otherwise,
        // we need to find the extreme point of the curve.
        if !bounds.contains(self.p1) {
            let (p0, p1, p2) = self.points_as_vec2s();
            let t = ((p0 - p1) / (p0 - 2.0 * p1 + p2)).clamp(0.0, 1.0);
            let s = Vec2::splat(1.0) - t;
            let p_extrema = s * s * p0 + 2.0 * s * t * p1 + t * t * p2;
            bounds = bounds.expand_to_include(p_extrema.into_point());
        }

        bounds
    }

    pub fn into_cubic_bezier(self) -> CubicBezier {
        CubicBezier {
            p0: self.p0,
            p1: eval_line(self.p0, self.p1, 1.0 / 3.0),
            p2: eval_line(self.p1, self.p2, 2.0 / 3.0),
            p3: self.p2,
        }
    }

    pub fn into_canonical_quad(self) -> CanonicalQuad {
        // Based on: https://astiopin.github.io/2019/01/04/qbez-parabola.html
        let (p0, p1, p2) = self.points_as_vec2s();

        // The parabola will have its y-axis tangent to p1 - (p0 + p1) / 2.0
        let mid = (p0 + p2) / 2.0;
        let y_vec = self.p1 - mid;
        todo!()
    }

    pub fn flatten(&self, sqrt_tolerance: f64, f: &mut impl FnMut(Line)) {}
}

/// Represents a QuadraticBezier that has been transformed into
/// a canonical parabola (y = x^2). Contains the transformation
/// and the end points.
pub struct CanonicalQuad {
    /// Unit vector in the parabolas y-direction
    pub y_vec: Vec2,
    /// Origin of the parabola
    pub origin: Vec2,

    pub scale: f64,
    /// Start point of the Bezier curve (in the parabolas coordinate system)
    pub x0: f64,
    /// End point of the Bezier curve (in the parabolas coordinate system)
    pub x2: f64,
}

impl CanonicalQuad {
    pub fn transform(&self) -> Transform {
        todo!()
    }

    pub fn closest_point_x(&self, p: Point) -> f64 {
        // Distance given by: |(x, x^2) - p|
        // Squaring and expanding gives:
        //   (x - px)^2 + (x^2 - py)^2 =
        //   x^4 + (1 - 2 * py) * x^2 + (-2 * px) * x + py^2 + px^2
        // The derivative is the depressed cubic:
        //   x^3 + (1/2 - py) * x - px / 2 = 0
        let dist_squared = Quartic::new([
            1.0,
            0.0,
            1.0 - 2.0 * p.y as f64,
            -2.0 * p.x as f64,
            p.distance_squared_to(&p),
        ]);
        let derivative = DepressedCubic::new(0.5 - p.y as f64, -0.5 * p.x as f64);
        let extremas = derivative.solve(self.x0..self.x2, 1.0e-8);

        let mut min_dist_squared = f64::min(
            dist_squared.value_at(self.x0),
            dist_squared.value_at(self.x2),
        );
        for extrema in extremas {
            min_dist_squared = min_dist_squared.min(dist_squared.value_at(extrema));
        }
        min_dist_squared.sqrt()
    }
}

#[inline(always)]
fn weighted_point_sum<const N: usize>(point_weights: [(f32, Point); N]) -> Point {
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
    pub fn new(p0: Point, p1: Point, p2: Point, p3: Point) -> Self {
        Self { p0, p1, p2, p3 }
    }

    pub fn eval(&self, t: f32) -> Point {
        eval_cubic(self.p0, self.p1, self.p2, self.p3, t)
    }

    pub fn split(&self, t: f32) -> (Self, Self) {
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
