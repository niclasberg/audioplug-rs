#[derive(Debug, Clone, Copy, PartialEq, Hash)]
pub struct Id(pub usize);

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct IdPath(Vec<Id>);

impl IdPath {
    pub const fn root() -> Self {
        Self(vec![Id(0)])
    }

    pub fn with_child_id<T>(&mut self, id: Id, f: impl FnOnce(&Self) -> T) -> T {
        self.0.push(id);
        let result = f(self);
        self.0.pop();
        result
    }
}