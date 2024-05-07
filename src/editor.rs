use crate::{view::View, Plugin, core::{Rectangle, Color, Point, Size}, view::Fill};

pub trait Editor {
    fn view(&self) -> impl View;
}

pub struct GenericEditor {

}

impl Editor for GenericEditor {
    fn view(&self) -> impl View {
        Rectangle::new(Point::ZERO, Size::new(200.0, 200.0)).fill(Color::RED)
    }
}