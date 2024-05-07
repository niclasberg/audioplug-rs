#[derive(Debug, Clone, Copy, PartialEq, Hash)]
pub struct Id(pub usize);

impl Id {
    pub fn next(&self) -> Self {
        Self(self.0 + 1)
    }
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct IdPath(Vec<Id>);

impl IdPath {
    pub fn root() -> Self {
        Self(vec![Id(0)])
    }

    pub fn push(&mut self, id: Id) {
        self.0.push(id);
    }

    pub fn pop(&mut self) {
        self.0.pop();
    }

	pub fn child_id(&self, id: Id) -> Self {
		let mut id_path = self.clone();
		id_path.0.push(id);
		id_path
	}

    pub fn next_sibling(&self) -> Self {
        let mut id_path = self.clone();
        let last = id_path.0.last_mut().unwrap();
        *last = last.next();
        id_path
    }

    pub fn with_child_id<T>(&mut self, id: Id, f: impl FnOnce(&Self) -> T) -> T {
        self.0.push(id);
        let result = f(self);
        self.0.pop();
        result
    }
}