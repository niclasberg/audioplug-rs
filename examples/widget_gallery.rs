use audioplug::App;
use audioplug::core::{Align, Color, ShadowKind, ShadowOptions, Size, Vec2};
use audioplug::ui::style::{ImageEffect, Length, UiRect};
use audioplug::ui::*;
use audioplug::views::*;
use std::path::Path;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Tab {
    Overview,
    Buttons,
}

fn main() {
    //let device = Device::new()?;
    //println!("name: {}, id: {}", device.name()?, device.id()?);

    let mut app = App::new();
    let _ = Window::open(
        &mut app,
        Stateful::new(|cx| {
            let tab = Var::new(cx, Tab::Overview);
            Row::new((
                Column::new((
                    menu_button("Overview", tab, Tab::Overview),
                    menu_button("Buttons", tab, Tab::Buttons),
                ))
                .style(|style, _| {
                    style.margin(UiRect::right_px(5.0));
                }),
                Switch::new(
                    move |cx| tab.get(cx),
                    move |tab| match tab {
                        Tab::Overview => overview().into_any_view(),
                        Tab::Buttons => buttons().into_any_view(),
                    },
                ),
            ))
            .style(|s, _| {
                s.background(Color::EARTH_YELLOW).width(Length::Vw(100.0));
            })
        }),
    );

    app.run();
}

fn menu_button(label: &str, tab_signal: Var<Tab>, tab: Tab) -> impl View {
    Button::new(Label::new(label))
        .on_click(move |cx| tab_signal.set(cx, tab))
        .style(move |style, _| {
            style.background(tab_signal.map(move |current_tab| {
                if *current_tab == tab {
                    Color::EARTH_YELLOW.tint(0.2)
                } else {
                    Color::EARTH_YELLOW
                }
                .into()
            }));
        })
}

fn overview() -> impl View {
    Stateful::new(|cx| {
        let checkbox_enabled = Var::new(cx, false);
        let text = Var::new(cx, "".to_string());
        let slider_value = Var::new(cx, 100.0);
        let checkbox_bg = AnimatedFn::tween(
            cx,
            move |cx| {
                let color = Color::MAY_GREEN;
                if checkbox_enabled.get(cx) {
                    color
                } else {
                    color.with_alpha(0.0)
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
            .style(|s, _| {
                s.border(Length::Px(2.0), Color::GRAY90)
                    .corner_radius(Size::new(2.0, 2.0))
                    .effects(vec![ImageEffect::GaussianBlur { radius: 10.0 }]);
            }),
            Row::new((
                Label::new("Slider"),
                Slider::new()
                    .range(1.0, 500.0)
                    .value(slider_value)
                    .on_value_changed(move |cx, value| slider_value.set(cx, value))
                    .style(|s, _| {
                        s.height(Length::Px(25.0));
                    }),
            ))
            .spacing(Length::Px(5.0))
            .v_align_center(),
            Row::new((Label::new("Knob"), Knob::new())).v_align_center(),
            Row::new((
                Label::new("Checkbox"),
                Checkbox::new()
                    .checked(checkbox_enabled)
                    .style(move |s, _| {
                        s.background(checkbox_bg.map(|c| Brush::Solid(*c)));
                    }),
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
                    .style(|s, _| {
                        s.corner_radius(Size::splat(5.0))
                            .background(Color::EARTH_YELLOW)
                            .box_shadow(ShadowOptions {
                                radius: 5.0,
                                ..Default::default()
                            })
                            .padding(UiRect::all_px(5.0));
                    })
                }),
            ))
            .v_align_center()
            .spacing(Length::Px(5.0)),
            Row::new((
                Label::new("Button"),
                Button::new(Label::new("Filled")).on_click(move |cx| {
                    checkbox_enabled.update(cx, |_, enabled| *enabled = !*enabled);
                }),
            ))
            .spacing(Length::Px(5.0))
            .v_align_center(),
            Row::new((
                Label::new("Image"),
                Image::from_file(Path::new("./ferris.png"))
                    .style(move |style, _| {
                        style
                            .max_width(Length::Px(200.0))
                            .height(animated.map(Length::from_px))
                            .corner_radius(Size::splat(7.0))
                            .box_shadow(ShadowOptions {
                                radius: 10.0,
                                offset: Vec2::splat(2.0),
                                color: Color::BLACK.with_alpha(0.3),
                                kind: ShadowKind::InnerShadow,
                                ..Default::default()
                            });
                    })
                    .overlay(
                        OverlayOptions {
                            align: Align::Bottom,
                            anchor: OverlayAnchor::OutsideParent,
                            ..Default::default()
                        },
                        Button::new(Label::new("Filled!!!")).on_click(move |cx| {
                            checkbox_enabled.update(cx, |_, enabled| *enabled = !*enabled);
                        }),
                    ),
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
