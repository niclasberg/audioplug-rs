use std::{any::Any, marker::PhantomData};

use crate::{
    app::{
        render::{fill_shape, stroke_shape},
        Accessor, AppState, BrushRef, BuildContext, EffectState, NodeId, Owner, ReactiveContext,
        ReadContext, RenderContext, Runtime, Scope, ShapeRef, TextLayout, View, Widget, WidgetId,
    },
    core::{Point, Rectangle},
    platform,
    style::DisplayStyle,
};

pub struct Canvas<FRender, State = ()> {
    f_render: FRender,
    state: PhantomData<State>,
}

impl<FRender, State> Canvas<FRender, State>
where
    State: 'static,
    FRender: Fn(&mut CanvasContext, Option<State>) -> State + 'static,
{
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

pub struct CanvasContext<'a, 'b> {
    effect_id: NodeId,
    widget_id: WidgetId,
    app_state: &'a mut AppState,
    renderer: platform::RendererRef<'b>,
}

impl<'a, 'b> CanvasContext<'a, 'b> {
    pub fn fill<'c, 'd>(&mut self, shape: impl Into<ShapeRef<'c>>, brush: impl Into<BrushRef<'d>>) {
        fill_shape(&mut self.renderer, shape.into(), brush.into());
    }

    pub fn stroke<'c, 'd>(
        &mut self,
        shape: impl Into<ShapeRef<'c>>,
        brush: impl Into<BrushRef<'d>>,
        line_width: f32,
    ) {
        stroke_shape(&mut self.renderer, shape.into(), brush.into(), line_width);
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

    pub fn draw_bitmap(&mut self, source: &platform::NativeImage, rect: impl Into<Rectangle>) {
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

impl<'a, 'b> ReactiveContext for CanvasContext<'a, 'b> {
    fn runtime(&self) -> &Runtime {
        self.app_state.runtime()
    }

    fn runtime_mut(&mut self) -> &mut Runtime {
        self.app_state.runtime_mut()
    }
}

impl<'a, 'b> ReadContext for CanvasContext<'a, 'b> {
    fn scope(&self) -> Scope {
        Scope::Node(self.effect_id)
    }
}

pub struct CanvasWidget<State> {
    effect_id: NodeId,
    state: Option<State>,
    f_render: Box<dyn Fn(&mut CanvasContext, Option<State>) -> State>,
}

impl<State: 'static> Widget for CanvasWidget<State> {
    fn display_style(&self) -> DisplayStyle {
        DisplayStyle::Block
    }

    fn debug_label(&self) -> &'static str {
        "Canvas"
    }

    fn render(&mut self, cx: &mut crate::app::RenderContext) {
        cx.app_state
            .runtime
            .subscriptions
            .clear_node_sources(self.effect_id);
        let mut cx = CanvasContext {
            widget_id: cx.id,
            effect_id: self.effect_id,
            app_state: &mut cx.app_state,
            renderer: cx.renderer,
        };

        let state = std::mem::take(&mut self.state);
        let new_state = (self.f_render)(&mut cx, state);
        self.state.replace(new_state);
    }
}
