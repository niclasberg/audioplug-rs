use crate::ui::{
    BuildContext, RenderContext, StyleExt, View, ViewStyle, Widget,
    reactive::{CanCreate, EffectState, NodeId, ReadScope},
};

type CanvasRenderFn = dyn FnMut(&mut RenderContext);

/// View that allows custom rendering.
pub struct Canvas<FRender> {
    style: ViewStyle,
    f_render: FRender,
}

impl<FRender> Canvas<FRender>
where
    FRender: FnMut(&mut RenderContext) + 'static,
{
    /// Create a Canvas, providing a function that performs rendering.
    ///
    /// # Example
    /// ```
    /// use crate::core::Color;
    /// let canvas = Canvas::new(move |cx, _| {
    ///     let bounds = cx.bounds();
    ///     cx.fill(bounds, Color::BLUE);
    /// })
    /// ```
    pub fn new(f_render: FRender) -> Self {
        Self {
            f_render,
            style: Default::default(),
        }
    }
}

impl<FRender> View for Canvas<FRender>
where
    FRender: FnMut(&mut RenderContext) + 'static,
{
    type Element = CanvasWidget;

    fn build(self, cx: &mut BuildContext<Self::Element>) -> Self::Element {
        let widget_id = cx.id();
        let effect_id = cx.create_context().create_effect_node(
            Box::new(move |cx| {
                cx.widget_mut(widget_id).request_render();
            }),
            false,
        );

        CanvasWidget {
            effect_id,
            f_render: Box::new(self.f_render),
        }
    }
}

impl<FRender> StyleExt for Canvas<FRender> {
    fn style_mut(&mut self) -> &mut ViewStyle {
        &mut self.style
    }
}

pub struct CanvasWidget {
    effect_id: NodeId,
    f_render: Box<CanvasRenderFn>,
}

impl Widget for CanvasWidget {
    fn debug_label(&self) -> &'static str {
        "Canvas"
    }

    fn render(&mut self, cx: &mut RenderContext) {
        cx.reactive_graph.clear_node_sources(self.effect_id);
        let old_scope = cx.read_scope;
        cx.read_scope = ReadScope::Node(self.effect_id);
        (self.f_render)(cx);
        cx.read_scope = old_scope;
    }
}
