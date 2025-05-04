use audioplug::app::*;
use audioplug::core::{Color, Size, SpringPhysics};
use audioplug::style::{Length, UiRect};
use audioplug::views::*;
use audioplug::App;
use std::path::Path;
use std::time::Duration;

fn main() {
    //let device = Device::new()?;
    //println!("name: {}, id: {}", device.name()?, device.id()?);

    let mut app = App::new();
    let _ = Window::open(
        &mut app,
        Scoped::new(|cx| {
            let checkbox_enabled = Signal::new(cx, false);
            let text = Signal::new(cx, "".to_string());
            let slider_value = Signal::new(cx, 100.0);
            let checkbox_bg = AnimatedFn::tween(
                cx,
                move |cx| {
                    if checkbox_enabled.get(cx) {
                        Color::MAY_GREEN
                    } else {
                        Color::FALU_RED
                    }
                },
                TweenOptions {
                    duration: Duration::from_secs_f64(0.4),
                    ..Default::default()
                },
            );

            let animated =
                AnimatedFn::spring(cx, move |cx| slider_value.get(cx), SpringOptions::default());

            Effect::new_with_state(cx, move |cx, cnt| {
                let cnt = cnt.unwrap_or(0);
                println!(
                    "Cnt: {}, Slider value: {}, enabled: {}",
                    cnt,
                    slider_value.get(cx),
                    checkbox_enabled.get(cx)
                );
                cnt + 1
            });

            Effect::watch(cx, slider_value, move |_, value| {
                println!("Effect::watch: slider_value: {}", value);
            });

            Column::new((
                Label::new(Computed::new(move |cx| {
                    format!(
                        "Slider value: {}, animated: {}",
                        slider_value.get(cx),
                        animated.get(cx)
                    )
                }))
                .style(|s| {
                    s.border(Length::Px(2.0), Color::GRAY90)
                        .corner_radius(Size::new(2.0, 2.0))
                }),
                Row::new((
                    Label::new("Slider"),
                    Slider::new()
                        .range(1.0, 500.0)
                        .value(slider_value)
                        .on_value_changed(move |cx, value| slider_value.set(cx, value))
                        .style(|s| s.height(Length::Px(25.0))),
                ))
                .spacing(Length::Px(5.0))
                .v_align_center(),
                Row::new((Label::new("Knob"), Knob::new())).v_align_center(),
                Row::new((
                    Label::new("Checkbox"),
                    Checkbox::new()
                        .checked(checkbox_enabled)
                        .style(|s| s.background(checkbox_bg.map(|c| Brush::Solid(*c)))),
                ))
                .v_align_center()
                .spacing(Length::Px(5.0)),
                Row::new((
                    Label::new("Button"),
                    Button::new(Label::new("Filled")).on_click(move |cx| {
                        checkbox_enabled.update(cx, |enabled| *enabled = !*enabled);
                    }),
                ))
                .spacing(Length::Px(5.0))
                .v_align_center(),
                Row::new((
                    Label::new("Image"),
                    Image::from_file(Path::new("./ferris.png"))
                        .style(|style| {
                            style
                                .max_width(Length::Px(200.0))
                                .height(animated.map(Length::from_px))
                        })
                        .overlay(UiRect::ZERO, Label::new("OVERLAY")),
                ))
                .v_align_center(),
                Row::new((
                    Label::new("Text input").color(Color::BLUE),
                    TextBox::new().on_input(move |cx, str| text.set(cx, str.to_string())),
                ))
                .spacing(Length::Px(5.0)),
            ))
            .spacing(Length::Px(5.0))
            .style(|s| {
                s.background(Color::EARTH_YELLOW)
                    .width(Length::Percent(100.0))
            })
        }),
    );

    app.run();
}
