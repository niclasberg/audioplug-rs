use std::any::Any;

use crate::{event::Event, core::{Constraint, Size, Color}}; 

mod view_node;
mod view_sequence;
pub use crate::id::IdPath;
pub use crate::{LayoutContext, EventContext, BuildContext, RenderContext};
pub use view_node::*;
pub use view_sequence::*;

#[derive(Debug, PartialEq)]
pub enum LayoutHint {
    Fixed,
    Flexible
}

impl LayoutHint {
    pub fn combine(&self, other: &Self) -> Self {
        match (self, other) {
            (LayoutHint::Fixed, LayoutHint::Fixed) => LayoutHint::Fixed,
            _ => LayoutHint::Flexible,
        }
    }
}

pub trait View: Sized {
    type Element: Widget;

    fn build(self, ctx: &mut BuildContext) -> Self::Element;

    fn background(self, color: Color) -> Background<Self> {
        Background { view: self, color }
    }
}

pub trait AnyView {
    fn dyn_build(&mut self, ctx: &mut BuildContext) -> Box<dyn Any>;
}

impl<V: View + 'static> AnyView for V {
    fn dyn_build(&mut self, ctx: &mut BuildContext) -> Box<dyn Widget> {
        Box::new(self.build(ctx))
    }
}

impl View for Box<dyn AnyView> {
    type Element = Box<dyn Widget>;

    fn build(&mut self, ctx: &mut BuildContext) -> Self::Element {
        self.deref_mut().dyn_build(ctx)
    }
}

pub trait Widget {
    fn event(&mut self, event: Event, ctx: &mut EventContext<()>);

    /// Layout the view and (possibly) its subviews
    /// The view is passed a constraint and returns the size it wants.
    fn layout(&mut self, constraint: Constraint, ctx: &mut LayoutContext) -> Size;

    /// Suggests how the size of the view is determined. 
    /// - A Fixed layout does not care about the suggested size passed into layout
    /// - The size of a Flexible layout depends on the size suggestion passed to layout
    fn layout_hint(&self) -> (LayoutHint, LayoutHint);
    fn render(&mut self, ctx: &mut RenderContext);
}

pub struct Background<V: View> {
    view: V,
    color: Color,
}

impl<V: View> View for Background<V> {
    type Element = BackgroundWidget<V::Element>;

    fn build(self, ctx: &mut BuildContext) -> Self::Element {
        BackgroundWidget { widget: self.view.build(ctx), color: self.color }
    }
}

pub struct BackgroundWidget<W: Widget> {
    widget: W,
    color: Color,
}

impl<W: View> Widget for BackgroundWidget<W> {
    fn event(&mut self, event: Event, ctx: &mut EventContext<()>) {
        self.widget.event(event, ctx)
    }

    fn layout(&mut self, constraint: Constraint, ctx: &mut LayoutContext) -> Size {
        self.widget.layout(constraint, ctx)
    }

    fn render(&mut self, ctx: &mut RenderContext) {
        ctx.fill(ctx.local_bounds(), self.color);
        self.widget.render(ctx)
    }

    fn layout_hint(&self) -> (LayoutHint, LayoutHint) {
        self.widget.layout_hint()
    }
} 