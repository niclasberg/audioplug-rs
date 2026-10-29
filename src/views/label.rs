use crate::ui::{
    BuildContext, EventHandlerExt, EventHandlers, Prop, StyleExt, TextExt, View, ViewExt,
    WidgetProps,
};

pub struct Label {
    props: WidgetProps,
    event_handlers: EventHandlers,
}

impl Label {
    pub fn new(str: impl Into<Prop<String>>) -> Self {
        Self {
            props: WidgetProps::default().text(str.into()),
            event_handlers: Default::default(),
        }
    }
}

impl View for Label {
    fn build(self, ctx: &mut BuildContext) {
        ctx.props(self.props).debug_label("Label");
    }
}

impl ViewExt for Label {
    fn props_mut(&mut self) -> &mut WidgetProps {
        &mut self.props
    }
}
impl StyleExt for Label {}
impl TextExt for Label {}

impl EventHandlerExt for Label {
    fn event_handlers_mut(&mut self) -> &mut EventHandlers {
        &mut self.event_handlers
    }
}
