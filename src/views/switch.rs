use crate::ui::{
    BuildContext, View,
    prelude::CanRead,
    reactive::{Effect, ReadContext},
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
    type Element = V::Element;

    fn build(self, cx: &mut BuildContext<Self::Element>) -> Self::Element {
        let Self { f_value, f_view } = self;
        let mut value = f_value(&mut cx.read_context());
        let widget = f_view(&value).build(cx);
        let id = cx.id();

        Effect::new(cx, move |cx| {
            let new_value = f_value(&mut cx.read_context());
            if new_value != value {
                cx.widget_mut(id)
                    .replace(f_view(&new_value))
                    .request_layout();
                value = new_value;
            }
        });

        widget
    }
}
