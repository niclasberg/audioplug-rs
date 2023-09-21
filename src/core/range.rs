#[derive(Debug, Clone, Copy)]
pub struct Range<T = f64> {
    min: T,
    max: T
}

impl<T> Range<T> where T: PartialOrd + Copy {
    pub fn new(min: T, max: T) -> Self {
        assert!(min <= max, "Invalid range, min cannot be larger than max");
        Self { min, max }
    }

    pub fn min(&self) -> T {
        self.min
    }

    pub fn max(&self) -> T {
        self.max
    }

    pub fn contains(&self, value: T) -> bool {
        value >= self.min && value <= self.max
    }

    pub fn intersects(&self, other: &Self) -> bool {
        !((self.min > other.max) || (self.max < other.min))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn contains() {
        assert!(Range::new(0.0, 0.0).contains(0.0));
        assert!(!Range::new(0.0, 0.0).contains(0.1));
        assert!(Range::new(0.0, 1.0).contains(0.0));
        assert!(Range::new(0.0, 1.0).contains(1.0));
        assert!(Range::new(0.0, 1.0).contains(0.5));
        assert!(!Range::new(0.0, 1.0).contains(-0.1));
        assert!(!Range::new(0.0, 1.0).contains(1.1));
    }
}