use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq, Hash)]
pub struct Id(pub usize);

impl Id {
    pub fn next(&self) -> Self {
        Self(self.0 + 1)
    }
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct IdPath(VecDeque<Id>);

impl IdPath {
    pub fn root() -> Self {
        Self(VecDeque::from([Id(0)]))
    }

    pub fn pop_root(&mut self) -> Option<Id> {
        self.0.pop_front()
    }

    pub fn push_child(&mut self, id: Id) {
        self.0.push_back(id);
    }

    pub fn pop_child(&mut self) {
        self.0.pop_back();
    }

    pub fn next_sibling(&self) -> Self {
        let mut id_path = self.clone();
        let last = id_path.0.back_mut().unwrap();
        *last = last.next();
        id_path
    }

    pub fn with_child_id<T>(&mut self, id: Id, f: impl FnOnce(&Self) -> T) -> T {
        self.push_child(id);
        let result = f(self);
        self.pop_child();
        result
    }
}