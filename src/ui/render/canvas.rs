use crate::{
    core::Rect,
    ui::{
        BuildContext, RenderContext, Scene, View, Widget, WidgetId, Widgets,
        reactive::{
            CanCreate, CanRead, EffectState, NodeId, ReactiveGraph, ReadContext, ReadScope,
        },
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
        let effect_id = cx.create_context().create_effect_node(state, false);

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
        self.widgets.content_bounds(self.widget_id)
    }
}

impl<'s> CanRead<'s> for CanvasContext<'s> {
    fn read_context<'s2>(&'s2 mut self) -> ReadContext<'s2>
    where
        's: 's2,
    {
        ReadContext {
            widgets: self.widgets,
            reactive_graph: self.reactive_graph,
            scope: ReadScope::Node(self.effect_id),
        }
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
