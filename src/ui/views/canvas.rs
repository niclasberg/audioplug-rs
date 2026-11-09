use crate::ui::{BuildContext, RenderContext, StyleExt, View, ViewExt, WidgetProps};

/// View that allows custom rendering.
pub struct Canvas {
    props: WidgetProps,
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
            props: Default::default(),
        }
    }
}

impl View for Canvas {
    fn build(self, cx: &mut BuildContext) {
        cx.render(self.f_render).props(self.props);
    }
}

impl ViewExt for Canvas {
    fn props_mut(&mut self) -> &mut WidgetProps {
        &mut self.props
    }
}

impl StyleExt for Canvas {}
