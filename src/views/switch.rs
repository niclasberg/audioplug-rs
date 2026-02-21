use crate::ui::{
    BuildContext, View,
    reactive::{Effect, ReadContext},
};

pub struct Switch<FValue, FView> {
    f_value: FValue,
    f_view: FView,
}

impl<T, V, FValue, FView> Switch<FValue, FView>
where
    T: PartialEq,
    FValue: Fn(&mut dyn ReadContext) -> T,
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
    FValue: Fn(&mut dyn ReadContext) -> T + 'static,
    FView: Fn(&T) -> V + 'static,
    V: View,
{
    type Element = V::Element;

    fn build(self, cx: &mut BuildContext<Self::Element>) -> Self::Element {
        let Self { f_value, f_view } = self;
        let mut value = f_value(cx);
        let widget = f_view(&value).build(cx);
        let id = cx.id();

        Effect::new(cx, move |cx| {
            let new_value = f_value(cx);
            if new_value != value {
                cx.replace_widget_dyn(id.id, f_view(&new_value).into_any_view());
                cx.widget_mut(id).request_layout();
                value = new_value;
            }
        });

        widget
    }
}
