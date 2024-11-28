use crate::app::Accessor;

pub struct StyleBuilder {
    pub(crate) hidden: Option<Accessor<bool>>,
}

impl StyleBuilder {
    pub fn hidden(mut self, value: impl Into<Accessor<bool>>) -> Self {
        self.hidden = Some(value.into());
        self
    }
}