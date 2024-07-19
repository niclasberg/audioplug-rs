use super::PlainValue;


pub struct StringListParameter {
    name: &'static str,
    strings: Vec<String>,
    default_index: usize,
}

impl StringListParameter {
    pub fn new(name: &'static str, strings: impl Into<Vec<String>>, default_index: usize) -> Self {
        Self {
            name, 
            strings: strings.into(),
            default_index
        }
    }

	pub fn name(&self) -> &'static str {
		self.name
	}

    pub fn string_count(&self) -> usize {
        self.strings.len()
    }

    pub fn step_count(&self) -> usize {
        if self.strings.is_empty() {
            0
        } else {
            self.strings.len() - 1
        }
    }

	pub fn default_value(&self) -> PlainValue {
		PlainValue::new(self.default_index as f64)
	}

    pub fn index_of(&self, key: &str) -> Option<usize> {
        self.strings.iter().position(|x| x == key)
    }
}
