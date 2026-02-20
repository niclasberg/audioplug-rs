use crate::{
    core::Rect,
    ui::{
        BuildContext, CreateContext, NodeId, ReactiveContext, ReactiveGraph, ReadContext,
        ReadScope, RenderContext, Scene, View, Widget, WidgetId, Widgets, reactive::EffectState,
        style::LayoutMode,
    },
};

type CanvasRenderFn = dyn FnMut(&mut CanvasContext) -> Scene;

/// View that allows custom rendering.
pub struct Canvas<FRender> {
    f_render: FRender,
}

impl<FRender> Canvas<FRender>
where
    FRender: FnMut(&mut CanvasContext) -> Scene + 'static,
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
        Self { f_render }
    }
}

impl<FRender> View for Canvas<FRender>
where
    FRender: FnMut(&mut CanvasContext) -> Scene + 'static,
{
    type Element = CanvasWidget;

    fn build(self, cx: &mut BuildContext<Self::Element>) -> Self::Element {
        let widget_id = cx.id();
        let state = EffectState::new(move |cx| {
            cx.widget_mut(widget_id).request_render();
        });
        let effect_id = (cx as &mut dyn CreateContext).create_effect_node(state, false);

        CanvasWidget {
            effect_id,
            f_render: Box::new(self.f_render),
        }
    }
}

pub struct CanvasContext<'a> {
    effect_id: NodeId,
    widget_id: WidgetId,
    reactive_graph: &'a mut ReactiveGraph,
    widgets: &'a Widgets,
}

impl CanvasContext<'_> {
    pub fn bounds(&self) -> Rect {
        self.widgets.get(self.widget_id).content_bounds()
    }
}

impl ReactiveContext for CanvasContext<'_> {
    fn reactive_graph_and_widgets(&self) -> (&ReactiveGraph, &Widgets) {
        (&self.reactive_graph, &self.widgets)
    }

    fn reactive_graph_mut_and_widgets(&mut self) -> (&mut ReactiveGraph, &Widgets) {
        (&mut self.reactive_graph, &self.widgets)
    }
}

impl ReadContext for CanvasContext<'_> {
    fn scope(&self) -> ReadScope {
        ReadScope::Node(self.effect_id)
    }
}

pub struct CanvasWidget {
    effect_id: NodeId,
    f_render: Box<CanvasRenderFn>,
}

impl Widget for CanvasWidget {
    fn layout_mode(&self) -> LayoutMode<'_> {
        LayoutMode::Block
    }

    fn debug_label(&self) -> &'static str {
        "Canvas"
    }

    fn render(&mut self, cx: &mut RenderContext) -> Scene {
        cx.reactive_graph.clear_node_sources(self.effect_id);
        let mut cx = CanvasContext {
            widget_id: cx.id,
            effect_id: self.effect_id,
            reactive_graph: cx.reactive_graph,
            widgets: cx.widgets,
        };

        (self.f_render)(&mut cx)
    }
}
