use std::{any::Any, marker::PhantomData};

use crate::{app::{Accessor, BuildContext, ReadContext, RenderContext, View, Widget}, style::DisplayStyle};

pub struct Canvas<T, FRender> {
    state: Accessor<T>,
    f_render: FRender,
}

impl<T, FRender> Canvas<T, FRender> 
where
    T: Clone + 'static,
    FRender: Fn(&mut RenderContext, &T) + 'static
{
    pub fn new(state: impl Into<Accessor<T>>, f_render: FRender) -> Self {
        let state = state.into();
        Self {
            state,
            f_render
        }
    }
}

impl<T, FRender> View for Canvas<T, FRender>
where
    T: Clone + 'static,
    FRender: Fn(&mut RenderContext, &T) + 'static
{
    type Element = CanvasWidget<T>;

    fn build(self, cx: &mut BuildContext<Self::Element>) -> Self::Element {
        CanvasWidget {
            state: self.state.get_and_bind(cx, |value, mut widget| {
                widget.state = value;
                widget.request_render();
            }),
            f_render: Box::new(self.f_render)
        }
    }
}

pub struct CanvasWidget<T> {
    state: T,
    f_render: Box<dyn Fn(&mut RenderContext, &T)>,
}

impl<T: Any> Widget for CanvasWidget<T> {
    fn display_style(&self) -> DisplayStyle {
        DisplayStyle::Block
    }

    fn debug_label(&self) -> &'static str {
        "Canvas"
    }

    fn render(&mut self, cx: &mut crate::app::RenderContext) {
        (self.f_render)(cx, &self.state)
    }
}