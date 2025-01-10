use audioplug::{app::{Signal, Window}, view::*, App};

struct Todo {
	index: usize,
	name: String,
	completed: bool,
}

fn main() {
	let mut app = App::new();
	let _ = Window::open(&mut app, |cx| {
		let items = Signal::new(cx, Vec::<Todo>::new());
		let text_input = Signal::new(cx, "".to_string());
		Flex::column((
			Label::new("TODO app"),
			TextBox::new()
				.on_input(move |cx, value| text_input.set(cx, value.to_string()))
		))
	});
	app.run();
}