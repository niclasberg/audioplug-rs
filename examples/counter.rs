use audioplug::{app::{Signal, SignalGet, SignalSet, Window}, core::Color, style::{Length, UiRect}, view::{Button, Flex, Label, View}, App};

fn main() {
    let mut app = App::new();
    let _ = Window::open(&mut app, |cx| {  
        let count = Signal::new(cx, 0);
        Flex::column((
            Label::new(count.map(|cnt| format!("Count is {}", cnt))),
			Button::new(Label::new("Increase"))
				.on_click(move |cx| {
					let old_value = count.get(cx);		
					count.set(cx, old_value + 1);
				}),
			Button::new(Label::new("Decrease"))
				.on_click(move |cx| {
					let old_value = count.get(cx);		
					count.set(cx, old_value - 1);
				}),
        ))
		.spacing(Length::Px(10.0))
		.style(|style| style
			.padding(UiRect::all_px(15.0))
			.background(count.map(|cnt| 
				if *cnt >= 0 {
					Color::from_rgb(0.8, 0.8, 0.8)
				} else {
					Color::RED
				}))
		)
    });
	app.run();
}