#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Size {
    pub width: f64,
    pub height: f64
}

impl Size {
    pub fn new(width: f64, height: f64) -> Self {
        Self { width, height }
    }
}

impl From<[u32; 2]> for Size {
    fn from([width, height]: [u32; 2]) -> Self {
        Self { width: width.into(), height: height.into() }
    }
}

impl From<[u16; 2]> for Size {
    fn from([width, height]: [u16; 2]) -> Self {
        Self { width: width.into(), height: height.into() }
    }
}