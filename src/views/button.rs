use crate::{
    core::{Color, Size},
    ui::{
        BuildContext, ButtonInteraction, EventContext, EventHandlerExt, EventHandlers, Interaction,
        Prop, StyleExt, TextExt, View, ViewExt, WidgetProps,
        reactive::{Computed, ReactiveValue, WidgetStatus},
        style::{Edges, Fill},
    },
};

pub struct LabelButton {
    props: WidgetProps,
    event_handlers: EventHandlers,
    kind: Interaction,
}

impl LabelButton {
    pub fn new(
        label: impl Into<Prop<String>>,
        click_fn: impl Fn(&mut EventContext) + 'static,
    ) -> Self {
        Self {
            props: WidgetProps::default()
                .text(label)
                .padding(Edges::all_px(5.0))
                .corner_radius(Size::splat(5.0))
                .background(Computed::new_const(|cx| {
                    let color = if WidgetStatus::PRESSED.get(cx) {
                        Color::from_rgb8(121, 153, 141)
                    } else {
                        Color::from_rgb8(101, 133, 121)
                    };
                    Fill::Solid(color)
                })),
            event_handlers: Default::default(),
            kind: Interaction::Button(ButtonInteraction::new(click_fn)),
        }
    }
}

impl ViewExt for LabelButton {
    fn props_mut(&mut self) -> &mut WidgetProps {
        &mut self.props
    }
}

impl StyleExt for LabelButton {}

impl TextExt for LabelButton {}

impl EventHandlerExt for LabelButton {
    fn event_handlers_mut(&mut self) -> &mut EventHandlers {
        &mut self.event_handlers
    }
}

impl View for LabelButton {
    fn build(self, cx: &mut BuildContext) {
        cx.props(self.props)
            .event_handlers(self.event_handlers)
            .interaction(self.kind);
    }
}

/*pub struct Button<V> {
    style: WidgetProps,
    child: V,
    click_fn: Box<ClickFn>,
}

impl<V: View> Button<V> {
    pub fn new(child: V, click_fn: impl FnMut(&mut EventContext) + 'static) -> Self {
        Self {
            style: default_style(),
            child,
            click_fn: Box::new(click_fn),
        }
    }
}

impl<V: View> View for Button<V> {
    fn build(self, cx: &mut BuildContext) {
        cx.style(self.style);
        cx.add_child(self.child);
    }
}
*/
