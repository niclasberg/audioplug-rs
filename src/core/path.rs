use crate::core::Point;

pub enum FlattenedPathElement {
    MoveTo(Point),
    LineTo(Point),
    ClosePath,
}

#[derive(Copy, Clone, Debug)]
pub enum PathElement {
    MoveTo(Point),
    LineTo(Point),
    QuadTo(Point, Point),
    CurveTo(Point, Point, Point),
    ClosePath,
}

pub struct Path {
    elements: Vec<PathElement>,
}

impl Path {
    pub const fn new() -> Self {
        Self {
            elements: Vec::new(),
        }
    }

    pub fn move_to(&mut self, p: Point) {
        self.elements.push(PathElement::MoveTo(p));
    }

    pub fn line_to(&mut self, p: Point) {
        self.elements.push(PathElement::LineTo(p));
    }

    /// Flattens the path into path elements corresponding to lines
    ///
    /// Adapted from Kurbo and (https://raphlinus.github.io/graphics/curves/2019/12/23/flatten-quadbez.html)
    pub fn flatten(&self, tolerance: f64, f: impl Fn(FlattenedPathElement)) {
        let mut last_point = None;
        let sqrt_tolerance = tolerance.sqrt();
        for &el in self.elements.iter() {
            match el {
                PathElement::MoveTo(point) => {
                    f(FlattenedPathElement::MoveTo(point));
                    last_point = Some(point);
                }
                PathElement::LineTo(point) => {
                    f(FlattenedPathElement::LineTo(point));
                    last_point = Some(point);
                }
                PathElement::QuadTo(p1, p2) => {
                    if let Some(p0) = last_point {
                        QuadBezier { p0, p1, p2 }.flatten(sqrt_tolerance, &f);
                    }
                    last_point = Some(p2);
                }
                PathElement::CurveTo(point, point1, point2) => todo!(),
                PathElement::ClosePath => {
                    last_point = None;
                    f(FlattenedPathElement::ClosePath);
                }
            }
        }
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

pub struct Line {
    p0: Point,
    p1: Point,
}

impl Line {
    pub fn eval(&self, t: f64) -> Point {
        Point::new(
            (1.0 - t) * self.p0.x + t * self.p1.x,
            (1.0 - t) * self.p0.y + t * self.p1.y,
        )
    }
}

pub struct QuadBezier {
    p0: Point,
    p1: Point,
    p2: Point,
}

impl QuadBezier {
    pub fn new(p0: Point, p1: Point, p2: Point) -> Self {
        Self { p0, p1, p2 }
    }

    pub fn eval(&self, t: f64) -> Point {
        let u = 1.0 - t;
        let w0 = u * u;
        let w1 = 2.0 * t * u;
        let w2 = t * t;
        Point::new(
            w0 * self.p0.x + w1 * self.p1.x + w2 * self.p2.x,
            w0 * self.p0.y + w1 * self.p1.y + w2 * self.p2.y,
        )
    }

    pub fn flatten(&self, sqrt_tolerance: f64, f: impl Fn(FlattenedPathElement)) {}
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

pub struct CubicBezier {
    p0: Point,
    p1: Point,
    p2: Point,
    p3: Point,
}

impl CubicBezier {
    pub fn eval(&self, t: f64) -> Point {
        let u = 1.0 - t;
        let w0 = u * u * u;
        let w1 = 3.0 * t * u * u;
        let w2 = 3.0 * t * t * u;
        let w3 = t * t * t;
        Point::new(
            w0 * self.p0.x + w1 * self.p1.x + w2 * self.p2.x + w3 * self.p3.x,
            w0 * self.p0.y + w1 * self.p1.y + w2 * self.p2.y + w3 * self.p3.y,
        )
    }
}

struct Stoker {}
