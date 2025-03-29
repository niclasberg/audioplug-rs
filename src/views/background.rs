use crate::{
    app::{
        Accessor, BuildContext, EventContext, EventStatus, MouseEventContext, RenderContext,
        StatusChange, View, Widget,
    },
    core::{Border, Color, Cursor},
    style::DisplayStyle,
    KeyEvent, MouseEvent,
};

pub struct Background<V: View> {
    pub(super) view: V,
    pub(super) fill: Option<Accessor<Color>>,
    pub(super) border: Option<Accessor<Border>>,
}

impl<V: View> Background<V> {
    fn background(mut self, color: impl Into<Accessor<Color>>) -> Self {
        self.fill = Some(color.into());
        self
    }

    fn border(mut self, border: impl Into<Accessor<Border>>) -> Self {
        self.border = Some(border.into());
        self
    }
}

impl<V: View> View for Background<V> {
    type Element = BackgroundWidget<V::Element>;

    fn build(self, cx: &mut BuildContext<Self::Element>) -> Self::Element {
        let widget = cx.build(self.view);

        let fill = self.fill.map(|fill| {
            fill.get_and_bind(cx, |value, mut widget| {
                widget.fill = Some(value);
                widget.request_render();
            })
        });

        let border = self.border.map(|border| {
            border.get_and_bind(cx, |value, mut widget| {
                if !widget
                    .border
                    .is_some_and(|border| border.color == value.color)
                {
                    widget.request_render();
                }
                widget.border = Some(value);
            })
        });

        if let Some(border) = border {
            cx.update_style(|style| {});
        }

        BackgroundWidget {
            widget,
            fill,
            border,
        }
    }
}

pub struct BackgroundWidget<W> {
    widget: W,
    fill: Option<Color>,
    border: Option<Border>,
}

impl<W: Widget> Widget for BackgroundWidget<W> {
    fn debug_label(&self) -> &'static str {
        self.widget.debug_label()
    }

    fn mouse_event(&mut self, event: MouseEvent, ctx: &mut MouseEventContext) -> EventStatus {
        self.widget.mouse_event(event, ctx)
    }

    fn key_event(&mut self, event: KeyEvent, ctx: &mut EventContext) -> EventStatus {
        self.widget.key_event(event, ctx)
    }

    fn status_updated(&mut self, event: StatusChange, ctx: &mut EventContext) {
        self.widget.status_updated(event, ctx)
    }

    fn render(&mut self, ctx: &mut RenderContext) {
        if let Some(fill) = self.fill {
            ctx.fill(ctx.local_bounds(), fill);
        }
        if let Some(border) = self.border {
            ctx.stroke(ctx.local_bounds(), border.color, border.width as f32);
        }

        self.widget.render(ctx)
    }

    fn display_style(&self) -> DisplayStyle {
        self.widget.display_style()
    }

    fn inner_widget(&self) -> Option<&dyn Widget> {
        Some(&self.widget)
    }

    fn inner_widget_mut(&mut self) -> Option<&mut dyn Widget> {
        Some(&mut self.widget)
    }
}
