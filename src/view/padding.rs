use crate::{widget::Widget, core::{Size, Layout, Vector}};

pub struct Padding<W: Widget> {
    child: W,
    left: f64,
    right: f64,
    top: f64,
    bottom: f64
}

impl<W: Widget> Padding<W> {
    pub fn new(left: f64, right: f64, top: f64, bottom: f64, child: W) -> Self {
        Self {left, right, top, bottom, child }
    }

    pub fn all(padding: f64, child: W) -> Self {
        Self::new(padding, padding, padding, padding, child)
    }
}

impl<W: Widget> Widget for Padding<W> {
    fn layout(&self, constraint: crate::core::Constraint) -> Layout {
        let padding = Size::new(
            self.left + self.right,
            self.top + self.bottom
        );
        
        constraint.shrink(padding);
        let child_layout = self.child.layout(constraint)
            .offset(Vector::new(self.left, self.top));
        let size = child_layout.bounds.size() + padding;

        Layout::with_child(size, child_layout)
    }

    fn event(&mut self, event: crate::event::Event) {
        self.child.event(event);
    }

    fn render(&self, layout: Layout, ctx: &mut crate::window::Renderer) {
        self.child.render(layout, ctx);
    }
}