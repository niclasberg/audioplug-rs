pub struct LineSegment {
    p0: f32,
    p1: f32,
}

pub struct GpuScene {
    line_segments: Vec<LineSegment>,
}
