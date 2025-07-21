use audioplug::{
    ui::*,
    core::{Color, Key, Size, UnitPoint},
    style::{AlignSelf, Length, UiRect},
    views::*,
    App,
};

fn main() {
    let mut app = App::new();
    let _ = Window::open(
        &mut app,
        Stateful::new(|cx| {
            let count = Var::new(cx, 0);

            let trigger = Trigger::new(cx);
            Effect::new(cx, move |cx| {
                trigger.track(cx);
                println!("Count = {}", count.get(cx));
            });
            Container::new(
                Column::new((
                    Label::new(count.map(|cnt| format!("Count: {}", cnt))),
                    Button::new(Label::new("Increase"))
                        .on_click(move |cx| count.update(cx, |_, value| *value += 1)),
                    Button::new(Label::new("Decrease"))
                        .on_click(move |cx| count.update(cx, |_, value| *value -= 1)),
                    Button::new(Label::new("Trigger")).on_click(move |cx| trigger.notify(cx)),
                    Label::new("No children to show")
                        .style(|style| style.hidden(count.map(|x| *x > 0))),
                    Column::new(IndexedViewSeq::new(
                        count.map(|&x| x.max(0) as usize),
                        |i| Label::new(format!("Child {}", i + 1)),
                    )),
                ))
                .spacing(Length::Px(10.0))
                .style(|style| {
                    style
                        .width(Length::Percent(30.0))
                        .min_width(Length::Px(200.0))
                        .padding(UiRect::all_px(15.0))
                        .corner_radius(Size::new(10.0, 10.0))
                        .align_self(AlignSelf::Center)
                        .background(count.map(|cnt| {
                            if *cnt >= 0 {
                                Color::from_rgb(0.8, 0.8, 0.8)
                            } else {
                                Color::RED
                            }
                            .into()
                        }))
                }),
            )
            .style(|style| {
                style
                    .height(Length::Vh(100.0))
                    .width(Length::Vw(100.0))
                    .border(Length::Px(2.0), Color::RED)
                    .background(LinearGradient::new(
                        (Color::WHITE, Color::GRAY90),
                        UnitPoint::TOP_LEFT,
                        UnitPoint::BOTTOM_RIGHT,
                    ))
            })
            .on_key_event(move |cx, event| match event {
                audioplug::KeyEvent::KeyDown { key, .. } => match key {
                    Key::Up => {
                        count.update(cx, |_, value| *value += 1);
                        EventStatus::Handled
                    }
                    Key::Down => {
                        count.update(cx, |_, value| *value -= 1);
                        EventStatus::Handled
                    }
                    _ => EventStatus::Ignored,
                },
                _ => EventStatus::Ignored,
            })
        }),
    );
    app.run();
}
