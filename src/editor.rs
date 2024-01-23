use crate::{View, Plugin};

pub trait Editor {
    fn view(&self) -> impl View;
}

struct GenericEditor {

}

