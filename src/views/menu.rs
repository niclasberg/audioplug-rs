use crate::app::WriteContext;

pub struct MenuItem {
    label: String,
    children: Vec<Self>,
    action: Option<Box<dyn Fn(&mut dyn WriteContext)>>,
}

pub struct Menu {
    items: Vec<MenuItem>,
}

impl Menu {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }
}
