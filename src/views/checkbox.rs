use crate::{
    KeyEvent,
    core::{Color, Key, Rect, Size},
    ui::{
        BuildContext, Capabilities, Event, EventContext, EventResult, Prop, StyleExt, View,
        ViewStyle, WidgetHandler,
        reactive::{ReactiveValue, Var},
        style::{Border, Edges, Fill, Length},
    },
};

type OnClickFn = dyn Fn(&mut EventContext, bool);

pub struct Checkbox {
    style: ViewStyle,
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
            style: ViewStyle {
                width: Some(Prop::Const(Length::Px(12.0))),
                height: Some(Prop::Const(Length::Px(12.0))),
                border: Some(Prop::Const(Border {
                    fill: Fill::Solid(Color::BLACK),
                    width: Length::Px(1.0),
                })),
                aspect_ratio: Some(Prop::Const(1.0)),
                corner_radius: Some(Prop::Const(Size::splat(3.0))),
                padding: Some(Prop::Const(Edges::all_px(0.5))),
                ..ViewStyle::DEFAULT
            },
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

impl StyleExt for Checkbox {
    fn style_mut(&mut self) -> &mut ViewStyle {
        &mut self.style
    }
}

impl View for Checkbox {
    fn build(self, cx: &mut BuildContext) {
        if let Some(checked) = self.checked {
            // User controls the checked property, so we should not toggle ourselves
            build_checkbox(cx, self.style, checked.clone(), move |cx| {
                let new_value = !checked.get(cx);
                (self.click_fn)(cx, new_value);
            });
        } else {
            // We control the check property, we should toggle on click
            let checked = Var::new(cx, false);
            build_checkbox(cx, self.style, checked.into(), move |cx| {
                let new_value = !checked.get(cx);
                checked.set(cx, new_value);
                (self.click_fn)(cx, new_value);
            });
        };
    }
}

fn build_checkbox(
    cx: &mut BuildContext,
    style: ViewStyle,
    checked: Prop<bool>,
    on_change: impl Fn(&mut EventContext) + 'static,
) {
    cx.style(style).handler(
        Capabilities::FOCUSABLE | Capabilities::CLICKABLE,
        CheckboxHandler { checked, on_change },
    );
}

struct CheckboxHandler<FOnChange> {
    checked: Prop<bool>,
    on_change: FOnChange,
}

impl<FOnChange> WidgetHandler for CheckboxHandler<FOnChange>
where
    FOnChange: Fn(&mut EventContext) + 'static,
{
    fn event(&mut self, cx: &mut EventContext, event: &Event) -> EventResult {
        match event {
            Event::Click(_) => {
                (self.on_change)(cx);
                EventResult::Stop
            }
            Event::Key(key_event) => match key_event {
                KeyEvent::KeyDown {
                    key: Key::Enter, ..
                } => {
                    (self.on_change)(cx);
                    EventResult::Stop
                }
                _ => EventResult::Continue,
            },
            _ => EventResult::Continue,
        }
    }

    fn render(&mut self, cx: &mut crate::ui::RenderContext) {
        if self.checked.get(cx) {
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
    }
}
