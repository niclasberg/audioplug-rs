use crate::{event::Event, window::Renderer, core::Rectangle};



pub trait Widget {
    fn layout(&self, bounds: Rectangle);
    fn event(&mut self, event: Event);
    fn render(&self, ctx: &mut Renderer);
}

