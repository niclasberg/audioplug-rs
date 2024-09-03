use audioplug::{app::{SignalGet, SignalSet, Window}, view::{Button, Column, Label}, App};

fn main() {
    let mut app = App::new();
    let _ = Window::open(&mut app, |ctx| {  
        let count = ctx.create_signal(0);
        let text = count.map(|cnt| format!("Count is {}", cnt));
        Column::new((
            Label::new("hi"),
			Button::new(Label::new("Increase"))
				.on_click({
					let count = count.clone();
					move |ctx| {
						let old_value = count.get(ctx);		
						count.set(ctx, old_value + 1);
					}
				}),
			Button::new(Label::new("Decrease"))
				.on_click({
					let count = count.clone();
					move |ctx| {
						let old_value = count.get(ctx);		
						count.set(ctx, old_value - 1);
					}
				}),
        ))
    });
	app.run();
}