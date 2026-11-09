use crate::ui::{
    BuildContext, View,
    prelude::CanRead,
    reactive::{Cached, Effect, ReadContext, Readable},
};

pub struct Switch<FValue, FView> {
    f_value: FValue,
    f_view: FView,
}

impl<T, V, FValue, FView> Switch<FValue, FView>
where
    T: PartialEq,
    FValue: Fn(&mut ReadContext) -> T,
    FView: Fn(&T) -> V,
    V: View,
{
    pub fn new(f_value: FValue, f_view: FView) -> Self {
        Self { f_value, f_view }
    }
}

impl<T, V, FValue, FView> View for Switch<FValue, FView>
where
    T: PartialEq + 'static,
    FValue: Fn(&mut ReadContext) -> T + 'static,
    FView: Fn(&T) -> V + 'static,
    V: View,
{
    fn build(self, cx: &mut BuildContext) {
        let Self { f_value, f_view } = self;
        let value = Cached::new(cx, move |cx, _| f_value(&mut cx.read_context()));
        let id = cx.id();

        Effect::new(cx, move |cx| {
            let view = value.with_ref(cx, &f_view);
            cx.ui.replace_widget(id, Some(cx.effect_id), view);
        });
    }
}
