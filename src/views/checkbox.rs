use crate::{
    core::{Color, Rect, Size},
    ui::{
        BuildContext, EventContext, Prop, StyleExt, View, ViewExt, WidgetProps,
        reactive::{ReactiveValue, Var},
        style::{Border, Edges, Fill, Length},
    },
};

type OnClickFn = dyn Fn(&mut EventContext, bool);

pub struct Checkbox {
    props: WidgetProps,
    checked: Option<Prop<bool>>,
    enabled: Prop<bool>,
    click_fn: Box<OnClickFn>,
}

impl Checkbox {
    pub fn new(f: impl Fn(&mut EventContext, bool) + 'static) -> Self {
        Self {
            checked: None,
            enabled: Prop::Const(true),
            click_fn: Box::new(f),
            props: WidgetProps::default()
                .width(Length::Px(12.0))
                .height(Length::Px(12.0))
                .border(Border {
                    fill: Fill::Solid(Color::BLACK),
                    width: Length::Px(1.0),
                })
                .aspect_ratio(1.0)
                .corner_radius(Size::splat(3.0))
                .padding(Edges::all_px(0.5)),
        }
    }

    pub fn checked(mut self, val: impl Into<Prop<bool>>) -> Self {
        self.checked = Some(val.into());
        self
    }

    pub fn enabled(mut self, val: impl Into<Prop<bool>>) -> Self {
        self.enabled = val.into();
        self
    }
}

impl ViewExt for Checkbox {
    fn props_mut(&mut self) -> &mut WidgetProps {
        &mut self.props
    }
}
impl StyleExt for Checkbox {}

impl View for Checkbox {
    fn build(self, cx: &mut BuildContext) {
        cx.props(self.props);
        let checked = if let Some(checked) = self.checked {
            // User controls the checked property, so we should not toggle ourselves
            let _checked = checked.clone();
            cx.button(move |cx| {
                let new_value = !checked.get(cx);
                (self.click_fn)(cx, new_value);
            });
            _checked
        } else {
            // We control the check property, we should toggle on click
            let checked = Var::new(cx, false);
            cx.button(move |cx| {
                let new_value = !checked.get(cx);
                checked.set(cx, new_value);
                (self.click_fn)(cx, new_value);
            });
            Prop::from(checked)
        };

        cx.render(move |cx| {
            if checked.get(cx) {
                let size = (cx.bounds().size().min_element() - 1.0).max(0.0);
                let bounds = Rect::from_center(cx.bounds().center(), Size::splat(size));
                cx.stroke_path(
                    |path| {
                        path.move_to(bounds.get_relative_point(0.05, 0.5))
                            .line_to(bounds.get_relative_point(0.35, 0.95))
                            .quad_to(
                                bounds.get_relative_point(0.5, 0.5),
                                bounds.get_relative_point(0.95, 0.05),
                            );
                    },
                    Color::BLACK,
                    2.0,
                )
            }
        });
    }
}
