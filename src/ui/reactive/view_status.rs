use crate::ui::{
    Prop, WidgetId, Widgets,
    reactive::{CanCreate, CanRead, Effect, EffectContext, ReactiveValue, ReadSignal},
};
use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct WidgetStatusFlags: u32 {
        const FOCUSED = 1 << 0;
        const HOVERED = 1 << 1;
        const CLICKED = 1 << 2;
    }
}

pub struct WidgetStatus<T> {
    pub mask: WidgetStatusFlags,
    pub getter: fn(&Widgets, WidgetId) -> T,
}

impl<T> Clone for WidgetStatus<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for WidgetStatus<T> {}

impl WidgetStatus<bool> {
    pub const FOCUSED: Self = WidgetStatus {
        mask: WidgetStatusFlags::FOCUSED,
        getter: |widgets, widget_id| widgets.has_focus(widget_id),
    };

    pub const PRESSED: Self = WidgetStatus {
        mask: WidgetStatusFlags::CLICKED,
        getter: |widgets, widget_id| widgets.is_pressed(widget_id),
    };

    pub const HOVERED: Self = WidgetStatus {
        mask: WidgetStatusFlags::HOVERED,
        getter: |widgets, widget_id| widgets.is_hovered(widget_id),
    };
}

impl<T> From<WidgetStatus<T>> for Prop<T> {
    fn from(value: WidgetStatus<T>) -> Self {
        Prop::ReadSignal(ReadSignal::from_widget_status(value))
    }
}

impl<T: 'static> ReactiveValue for WidgetStatus<T> {
    type Value = T;

    fn track<'s>(&self, cx: &mut impl CanRead<'s>) {
        cx.read_context().track_widget_status(self.mask)
    }

    fn with_ref_untracked<'s, R>(
        &self,
        cx: &mut impl CanRead<'s>,
        f: impl FnOnce(&Self::Value) -> R,
    ) -> R {
        let read_cx = cx.read_context();
        let widget_id = read_cx
            .current_widget
            .expect("View status can only be read when accessed from a View");

        let value = (self.getter)(read_cx.widgets, widget_id);
        f(&value)
    }

    fn watch<'s, F>(self, cx: &mut impl CanCreate<'s>, f: F) -> Effect
    where
        F: FnMut(&mut EffectContext, &Self::Value) + 'static,
    {
        let create_cx = cx.create_context();
        let widget_id = create_cx
            .owning_widget()
            .expect("ViewStatus can only be watched if the effect is owned by a widget");
        Effect::watch_widget_status(create_cx, widget_id, self, f)
    }
}
