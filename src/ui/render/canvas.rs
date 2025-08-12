use crate::{
    core::Rect,
    platform,
    ui::{
        AppState, BuildContext, NodeId, Owner, ReactiveContext, ReadContext, ReadScope, Scene,
        View, Widget, WidgetId,
        reactive::{EffectState, create_effect_node},
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
        let effect_id = create_effect_node(
            &mut cx.as_create_context(Owner::Widget(widget_id.id)),
            state,
            false,
        );

        CanvasWidget {
            effect_id,
            f_render: Box::new(self.f_render),
        }
    }
}

pub struct CanvasContext<'a> {
    effect_id: NodeId,
    widget_id: WidgetId,
    app_state: &'a mut AppState,
}

impl CanvasContext<'_> {
    pub fn bounds(&self) -> Rect {
        self.app_state
            .widget_data_ref(self.widget_id)
            .content_bounds()
    }
}

impl ReactiveContext for CanvasContext<'_> {
    fn app_state(&self) -> &AppState {
        self.app_state
    }

    fn app_state_mut(&mut self) -> &mut AppState {
        self.app_state
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
    fn layout_mode(&self) -> LayoutMode {
        LayoutMode::Block
    }

    fn debug_label(&self) -> &'static str {
        "Canvas"
    }

    fn render(&mut self, cx: &mut crate::ui::RenderContext) -> Scene {
        cx.app_state.runtime.clear_node_sources(self.effect_id);
        let mut cx = CanvasContext {
            widget_id: cx.id,
            effect_id: self.effect_id,
            app_state: cx.app_state,
        };

        (self.f_render)(&mut cx)
    }
}
