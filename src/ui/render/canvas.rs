use std::marker::PhantomData;

use crate::{
    core::{Point, Rectangle},
    platform,
    ui::{
        AppState, BrushRef, BuildContext, EffectState, NodeId, Owner, ReactiveContext,
        ReactiveGraph, ReadContext, Scope, ShapeRef, TextLayout, View, Widget, WidgetId,
        style::LayoutMode, widget_status::WidgetStatus,
    },
};

/// View that allows custom rendering.
pub struct Canvas<FRender, State = ()> {
    f_render: FRender,
    state: PhantomData<State>,
}

impl<FRender, State> Canvas<FRender, State>
where
    State: 'static,
    FRender: Fn(&mut CanvasContext, Option<State>) -> State + 'static,
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
            state: PhantomData,
        }
    }
}

impl<FRender, State> View for Canvas<FRender, State>
where
    State: 'static,
    FRender: Fn(&mut CanvasContext, Option<State>) -> State + 'static,
{
    type Element = CanvasWidget<State>;

    fn build(self, cx: &mut BuildContext<Self::Element>) -> Self::Element {
        let widget_id = cx.id();

        let state = EffectState::new(move |cx| {
            cx.widget_mut(widget_id).request_render();
        });
        let effect_id =
            cx.runtime_mut()
                .create_effect_node(state, Some(Owner::Widget(widget_id.id)), false);

        CanvasWidget {
            effect_id,
            state: None,
            f_render: Box::new(self.f_render),
        }
    }
}

pub struct CanvasContext<'a, 'b, 'c> {
    effect_id: NodeId,
    widget_id: WidgetId,
    app_state: &'a mut AppState,
    renderer: &'b mut platform::RendererRef<'c>,
}

impl CanvasContext<'_, '_, '_> {
    pub fn fill<'a, 'b>(&mut self, shape: impl Into<ShapeRef<'a>>, brush: impl Into<BrushRef<'b>>) {
        self.renderer.fill_shape(shape.into(), brush.into());
    }

    pub fn stroke<'c, 'd>(
        &mut self,
        shape: impl Into<ShapeRef<'c>>,
        brush: impl Into<BrushRef<'d>>,
        line_width: f32,
    ) {
        self.renderer
            .stroke_shape(shape.into(), brush.into(), line_width);
    }

    pub fn draw_line<'c>(
        &mut self,
        p0: Point,
        p1: Point,
        brush: impl Into<BrushRef<'c>>,
        line_width: f32,
    ) {
        self.renderer.draw_line(p0, p1, brush.into(), line_width)
    }

    pub fn draw_lines<'c>(
        &mut self,
        points: &[Point],
        brush: impl Into<BrushRef<'c>>,
        line_width: f32,
    ) {
        let brush = brush.into();
        for p in points.windows(2) {
            self.renderer.draw_line(p[0], p[1], brush, line_width)
        }
    }

    pub fn draw_bitmap(&mut self, source: &platform::Bitmap, rect: impl Into<Rectangle>) {
        self.renderer.draw_bitmap(source, rect.into())
    }

    pub fn draw_text(&mut self, text_layout: &TextLayout, position: Point) {
        self.renderer.draw_text(&text_layout.0, position)
    }

    pub fn bounds(&self) -> Rectangle {
        self.app_state
            .widget_data_ref(self.widget_id)
            .content_bounds()
    }
}

impl ReactiveContext for CanvasContext<'_, '_, '_> {
    fn runtime(&self) -> &ReactiveGraph {
        self.app_state.runtime()
    }

    fn runtime_mut(&mut self) -> &mut ReactiveGraph {
        self.app_state.runtime_mut()
    }

    fn widget_status(&self, widget_id: WidgetId) -> Option<WidgetStatus> {
        self.app_state.widget_status(widget_id)
    }
}

impl ReadContext for CanvasContext<'_, '_, '_> {
    fn scope(&self) -> Scope {
        Scope::Node(self.effect_id)
    }
}

type CanvasRenderFn<State> = dyn Fn(&mut CanvasContext, Option<State>) -> State;

pub struct CanvasWidget<State> {
    effect_id: NodeId,
    state: Option<State>,
    f_render: Box<CanvasRenderFn<State>>,
}

impl<State: 'static> Widget for CanvasWidget<State> {
    fn layout_mode(&self) -> LayoutMode {
        LayoutMode::Block
    }

    fn debug_label(&self) -> &'static str {
        "Canvas"
    }

    fn render(&mut self, cx: &mut crate::ui::RenderContext) {
        cx.app_state
            .runtime
            .subscriptions
            .clear_node_sources(self.effect_id);
        let mut cx = CanvasContext {
            widget_id: cx.id,
            effect_id: self.effect_id,
            app_state: cx.app_state,
            renderer: &mut cx.renderer,
        };

        let state = std::mem::take(&mut self.state);
        let new_state = (self.f_render)(&mut cx, state);
        self.state.replace(new_state);
    }
}
