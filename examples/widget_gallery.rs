use audioplug::core::{Color, Size, Vec2};
use audioplug::ui::reactive::{SpringOptions, TweenOptions};
use audioplug::ui::style::{Border, Edges, Fill, Length};
use audioplug::ui::{App, Paint, ShadowKind, ShadowOptions, TextExt, Window};
use audioplug::ui::{StyleExt, prelude::*};
use audioplug::views::*;
use std::path::Path;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Tab {
    Overview,
    Buttons,
}

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();
    //let device = Device::new()?;
    //println!("name: {}, id: {}", device.name()?, device.id()?);

    let mut app = App::new();
    let _window = Window::open(
        &mut app,
        Stateful::new(|cx| {
            let tab = Var::new(cx, Tab::Overview);
            Row::new((
                Column::new((
                    menu_button("Overview", tab, Tab::Overview),
                    menu_button("Buttons", tab, Tab::Buttons),
                ))
                .margin(Edges::right_px(5.0)),
                Switch::new(
                    move |cx| tab.get(cx),
                    move |tab| match tab {
                        Tab::Overview => overview().into_any_view(),
                        Tab::Buttons => buttons().into_any_view(),
                    },
                ),
            ))
            .background(Color::EARTH_YELLOW)
            .width(Length::Vw(100.0))
        }),
    );

    app.run();
}

fn menu_button(label: &str, tab_signal: Var<Tab>, tab: Tab) -> impl View {
    LabelButton::new(label, move |cx| tab_signal.set(cx, tab)).background(tab_signal.map(
        move |current_tab| {
            if *current_tab == tab {
                Color::EARTH_YELLOW.tint(0.2)
            } else {
                Color::EARTH_YELLOW
            }
            .into()
        },
    ))
}

fn overview() -> impl View {
    Stateful::new(|cx| {
        let checkbox_enabled = Var::new(cx, false);
        let text = Var::new(cx, "".to_string());
        let slider_value = Var::new(cx, 100.0);
        let checkbox_bg = Animated::tween(
            cx,
            move |cx| {
                let color = Color::MAY_GREEN;
                if checkbox_enabled.get(cx) {
                    color
                } else {
                    color.with_alpha(0.3)
                }
            },
            TweenOptions {
                duration: Duration::from_secs_f64(0.4),
                ..Default::default()
            },
        );

        let animated =
            Animated::spring(cx, move |cx| slider_value.get(cx), SpringOptions::default());

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

        slider_value.watch(cx, move |_, value| {
            println!("Effect::watch: slider_value: {value}");
        });

        Column::new((
            Label::new(Computed::new(move |cx| {
                format!(
                    "Slider value: {}, animated: {}",
                    slider_value.get(cx),
                    animated.get(cx)
                )
            }))
            .border(Border::new(Color::GRAY90, Length::Px(2.0)))
            .corner_radius(Size::new(2.0, 2.0)),
            Row::new((
                Label::new("Slider"),
                Slider::new(move |cx, value| slider_value.set(cx, value))
                    .range(1.0, 500.0)
                    .value(slider_value)
                    .max_width(Length::Px(200.0))
                    .flex_grow(1.0),
            ))
            .spacing(Length::Px(5.0))
            .v_align_center(),
            Row::new((Label::new("Knob"), Knob::new())).v_align_center(),
            Row::new((
                Label::new("Checkbox"),
                Checkbox::new()
                    .checked(checkbox_enabled)
                    .background(checkbox_bg.map(|c| Fill::Solid(*c))),
            ))
            .v_align_center()
            .spacing(Length::Px(5.0)),
            Row::new((
                Label::new("Dropdown"),
                Dropdown::new(Label::new("Select"), move || {
                    Column::new((
                        Label::new("Open..."),
                        Label::new("Close..."),
                        Label::new("Eat banana"),
                    ))
                    .spacing(Length::Px(2.5))
                    .corner_radius(Size::splat(5.0))
                    .background(Color::EARTH_YELLOW)
                    .box_shadow(ShadowOptions {
                        radius: 5.0,
                        ..Default::default()
                    })
                    .padding(Edges::all_px(5.0))
                }),
            ))
            .v_align_center()
            .spacing(Length::Px(5.0)),
            Row::new((
                Label::new("Button"),
                LabelButton::new("Filled", move |cx| {
                    checkbox_enabled.update(cx, |_, enabled| *enabled = !*enabled);
                })
                .font_size(22.0),
            ))
            .spacing(Length::Px(5.0))
            .v_align_center(),
            Row::new((
                Label::new("Image"),
                Image::from_file(Path::new("./ferris.png"))
                    .max_width(Length::Px(200.0))
                    .height(animated.map(|a| Length::Px(*a as f32)))
                    .corner_radius(Size::splat(7.0))
                    .box_shadow(ShadowOptions {
                        radius: 10.0,
                        offset: Vec2::splat(2.0),
                        color: Color::BLACK.with_alpha(0.3),
                        kind: ShadowKind::InnerShadow,
                    }), /*.overlay(
                            OverlayOptions {
                                align: Align::Bottom,
                                anchor: OverlayAnchor::OutsideParent,
                                ..Default::default()
                            },
                            Button::new_with_label("Filled!!!", move |cx| {
                                checkbox_enabled.update(cx, |_, enabled| *enabled = !*enabled);
                            }),
                        ), */
            ))
            .v_align_center(),
            Row::new((
                Label::new("Text input").color(Color::BLUE),
                TextBox::new(move |cx, str| text.set(cx, str.to_string())),
            ))
            .spacing(Length::Px(5.0)),
        ))
        .spacing(Length::Px(5.0))
    })
}

pub fn buttons() -> impl View {
    Label::new("Buttons")
}
