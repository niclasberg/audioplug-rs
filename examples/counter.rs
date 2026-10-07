use audioplug::{
    core::{Color, Key, Size, UnitPoint},
    ui::{
        App, StyleExt, Window,
        prelude::*,
        style::{AlignSelf, Border, Edges, Fill, Length},
    },
    views::*,
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
                    Label::new(count.map(|cnt| format!("Count: {cnt}"))),
                    LabelButton::new("Increase", move |cx| {
                        count.update(cx, |_, value| *value += 1)
                    }),
                    LabelButton::new("Decrease", move |cx| {
                        count.update(cx, |_, value| *value -= 1)
                    }),
                    LabelButton::new("Trigger", move |cx| trigger.notify(cx)),
                    Label::new("No children to show").hidden(count.map(|x| *x > 0)),
                    Column::new(IndexedViewSeq::new(
                        count.map(|&x| x.max(0) as usize),
                        |i| Label::new(format!("Child {}", i + 1)),
                    )),
                ))
                .spacing(Length::Px(10.0))
                .width(Length::Percent(30.0))
                .min_width(Length::Px(200.0))
                .padding(Edges::all_px(15.0))
                .corner_radius(Size::new(10.0, 10.0))
                .align_self(AlignSelf::CENTER)
                .background(count.map(|cnt| {
                    if *cnt >= 0 {
                        Color::from_rgb(0.8, 0.8, 0.8)
                    } else {
                        Color::RED
                    }
                    .into()
                })),
            )
            .height(Length::Vh(100.0))
            .width(Length::Vw(100.0))
            .border(Border::new(Color::RED, Length::Px(2.0)))
            .background(Fill::LinearGradient {
                colors: (Color::WHITE, Color::GRAY90).into(),
                start: UnitPoint::TOP_LEFT,
                end: UnitPoint::BOTTOM_RIGHT,
            })
            .on_key_event(move |cx, event| match event {
                audioplug::KeyEvent::KeyDown { key, .. } => match key {
                    Key::Up => {
                        count.update(cx, |_, value| *value += 1);
                        EventResult::Stop
                    }
                    Key::Down => {
                        count.update(cx, |_, value| *value -= 1);
                        EventResult::Stop
                    }
                    _ => EventResult::Continue,
                },
                _ => EventResult::Continue,
            })
        }),
    );
    app.run();
}
