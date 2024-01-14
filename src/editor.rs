use crate::View;

pub trait Editor {
    fn view(&self) -> impl View;
}