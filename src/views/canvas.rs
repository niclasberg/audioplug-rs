use crate::ui::{
    BuildContext, Capabilities, RenderContext, StyleExt, View, ViewStyle, WidgetHandler,
};

/// View that allows custom rendering.
pub struct Canvas {
    style: ViewStyle,
    f_render: Box<dyn FnMut(&mut RenderContext)>,
}

impl Canvas {
    /// Create a Canvas, providing a function that performs rendering.
    ///
    /// # Example
    /// ```
    /// use crate::core::Color;
    /// let canvas = Canvas::new(move |cx| {
    ///     let bounds = cx.bounds();
    ///     cx.fill(bounds, Color::BLUE);
    /// })
    /// ```
    pub fn new(f: impl FnMut(&mut RenderContext) + 'static) -> Self {
        Self {
            f_render: Box::new(f),
            style: Default::default(),
        }
    }
}

impl View for Canvas {
    fn build(self, cx: &mut BuildContext) {
        struct Handler {
            f_render: Box<dyn FnMut(&mut RenderContext)>,
        }
        impl WidgetHandler for Handler {
            fn event(
                &mut self,
                _cx: &mut crate::ui::EventContext,
                _event: &crate::ui::Event,
            ) -> crate::ui::prelude::EventResult {
                crate::ui::prelude::EventResult::Continue
            }

            fn render(&mut self, cx: &mut RenderContext) {
                (self.f_render)(cx)
            }
        }

        cx.handler(
            Capabilities::empty(),
            Handler {
                f_render: self.f_render,
            },
        )
        .style(self.style);
    }
}

impl StyleExt for Canvas {
    fn style_mut(&mut self) -> &mut ViewStyle {
        &mut self.style
    }
}
