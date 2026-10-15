use crate::ui::{
    BuildContext, EventHandlerExt, Prop, StyleExt, TextExt, View, ViewEventHandlers, ViewStyle,
    ViewText,
};

pub struct Label {
    style: ViewStyle,
    text: ViewText,
    event_handlers: ViewEventHandlers,
}

impl Label {
    pub fn new(str: impl Into<Prop<String>>) -> Self {
        Self {
            style: Default::default(),
            text: ViewText {
                text: Some(str.into()),
                ..Default::default()
            },
            event_handlers: Default::default(),
        }
    }
}

impl View for Label {
    fn build(self, ctx: &mut BuildContext) {
        ctx.style(self.style).text(self.text);
    }
}

impl StyleExt for Label {
    fn style_mut(&mut self) -> &mut ViewStyle {
        &mut self.style
    }
}

impl EventHandlerExt for Label {
    fn event_handlers_mut(&mut self) -> &mut ViewEventHandlers {
        &mut self.event_handlers
    }
}

impl TextExt for Label {
    fn text_mut(&mut self) -> &mut ViewText {
        &mut self.text
    }
}
