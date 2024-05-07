use std::ops::{Deref, DerefMut};

use crate::{event::Event, core::Color}; 

mod view_node;
mod view_sequence;
pub use crate::id::IdPath;
pub use view_node::*;
pub use view_sequence::*;
mod label;
//mod stack;
mod button;
mod linear_layout;
mod slider;
mod xy_pad;
mod textbox;
mod filled;
mod contexts;
mod styled;
mod scroll;

pub use button::Button;
pub use linear_layout::{Column, Row};
pub use label::Label;
pub use slider::Slider;
pub use xy_pad::XyPad;
pub use textbox::TextBox;
pub use filled::*;
pub use contexts::*;
pub use styled::*;
pub use scroll::*;

pub trait View: Sized {
    type Element: Widget + 'static;

    fn build(self, ctx: &mut BuildContext) -> Self::Element;

    fn background(self, color: Color) -> Background<Self> {
        Background { view: self, color }
    }

    fn with_style<F: Fn(&mut taffy::Style)>(self, f: F) -> Styled<Self, F> {
        Styled {
            view: self,
            style_function: f
        }
    }

    fn as_any(self) -> Box<dyn AnyView> 
    where 
        Self: 'static 
    {
        Box::new(self)
    }
}

impl<W: Widget + 'static, F: FnOnce(&mut BuildContext) -> W> View for F {
    type Element = W;

    fn build(self, ctx: &mut BuildContext) -> Self::Element {
        self(ctx)
    }
}

pub trait AnyView {
    fn dyn_build(self, ctx: &mut BuildContext) -> Box<dyn Widget>;
}

impl<V: View + 'static> AnyView for V {
    fn dyn_build(self, ctx: &mut BuildContext) -> Box<dyn Widget> {
        Box::new(self.build(ctx))
    }
}

impl View for Box<dyn AnyView> {
    type Element = Box<dyn Widget>;

    fn build(self, ctx: &mut BuildContext) -> Self::Element {
        self.dyn_build(ctx)
    }
}

pub trait Widget {
    fn event(&mut self, event: Event, ctx: &mut EventContext);
    fn layout(&mut self, inputs: taffy::LayoutInput, ctx: &mut LayoutContext) -> taffy::LayoutOutput;
    fn style(&self) -> taffy::Style;
    fn render(&mut self, ctx: &mut RenderContext);

    fn mouse_enter(&mut self, _ctx: &mut EventContext) { }
    fn mouse_exit(&mut self, _ctx: &mut EventContext) { }

    fn child_count(&self) -> usize { 0 }
    fn get_child<'a>(&'a self, _i: usize) -> &'a WidgetNode { unreachable!() }
    fn get_child_mut<'a>(&'a mut self, _i: usize) -> &'a mut WidgetNode { unreachable!() }
}

impl Widget for Box<dyn Widget> {
    fn event(&mut self, event: Event, ctx: &mut EventContext) {
        self.deref_mut().event(event, ctx)
    }

    fn layout(&mut self, inputs: taffy::LayoutInput, ctx: &mut LayoutContext) -> taffy::LayoutOutput {
        self.deref_mut().layout(inputs, ctx)
    }

    fn render(&mut self, ctx: &mut RenderContext) {
        self.deref_mut().render(ctx)
    }

    fn mouse_enter(&mut self, ctx: &mut EventContext) { 
        self.deref_mut().mouse_enter(ctx)
    }

    fn mouse_exit(&mut self, ctx: &mut EventContext) { 
        self.deref_mut().mouse_exit(ctx)
    }

    fn style(&self) -> taffy::Style {
        self.deref().style()
    }

    fn child_count(&self) -> usize { 
        self.deref().child_count()
    }

    fn get_child<'a>(&'a self, i: usize) -> &'a WidgetNode { 
        self.deref().get_child(i)
    }

    fn get_child_mut<'a>(&'a mut self, i: usize) -> &'a mut WidgetNode { 
        self.deref_mut().get_child_mut(i)
    }
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

pub struct BackgroundWidget<W> {
    widget: W,
    color: Color,
}

impl<W: Widget> Widget for BackgroundWidget<W> {
    fn event(&mut self, event: Event, ctx: &mut EventContext) {
        self.widget.event(event, ctx)
    }

    fn layout(&mut self, inputs: taffy::LayoutInput, ctx: &mut LayoutContext) -> taffy::LayoutOutput {
        self.widget.layout(inputs, ctx)
    }

    fn render(&mut self, ctx: &mut RenderContext) {
        ctx.fill(ctx.local_bounds(), self.color);
        self.widget.render(ctx)
    }

    fn mouse_enter(&mut self, ctx: &mut EventContext) { 
        self.widget.mouse_enter(ctx)
    }

    fn mouse_exit(&mut self, ctx: &mut EventContext) { 
        self.widget.mouse_exit(ctx)
    }

    fn style(&self) -> taffy::Style {
        self.widget.style()
    }

    fn child_count(&self) -> usize { 
        self.widget.child_count()
    }

    fn get_child<'a>(&'a self, i: usize) -> &'a WidgetNode { 
        self.widget.get_child(i)
    }

    fn get_child_mut<'a>(&'a mut self, i: usize) -> &'a mut WidgetNode { 
        self.widget.get_child_mut(i)
    }
} 
