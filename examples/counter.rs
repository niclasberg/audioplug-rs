use audioplug::{app::{Signal, SignalGet, SignalSet, Window}, core::Color, view::{Button, Column, Label, View}, App};

fn main() {
    let mut app = App::new();
    let _ = Window::open(&mut app, |cx| {  
        let count = Signal::new(cx, 0);
        Column::new((
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
		.spacing(5.0)
		.padding(15.0)
		.background(count.map(|cnt| 
			if *cnt >= 0 {
				Color::from_rgb(0.8, 0.8, 0.8)
			} else {
				Color::RED
			}))
    });
	app.run();
}