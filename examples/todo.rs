use std::sync::atomic::AtomicUsize;

use audioplug::{app::{Signal, SignalContext, SignalCreator, SignalGet, ViewContext, Window}, core::Color, style::Length, view::*, App};

struct Todos(pub Vec<Todo>);

#[derive(Clone, Copy)]
struct Todo {
	index: usize,
	name: Signal<String>,
	completed: Signal<bool>,
}

static NEXT_ID: AtomicUsize = AtomicUsize::new(0);
impl Todo {
	pub fn new(cx: &mut dyn SignalCreator, name: &str, completed: bool) -> Self {
		Self {
			index: NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
			name: Signal::new(cx, name.to_string()),
			completed: Signal::new(cx, completed),
		}
	}

	pub fn toggle(&self, cx: &mut impl SignalContext) {
		self.completed.update(cx, |value| *value = !*value);
	}
}

fn todo_view(todo: Todo) -> impl View {
	Flex::row((
		Label::new(todo.name),
		Button::new(Label::new("Remove"))
	))
	.style(|s| s.background(todo.completed.map(|&c| if c { Color::GREEN} else { Color::RED })))
}

fn main() {
	let mut app = App::new();
	let _ = Window::open(&mut app, |cx| {
		let items = Signal::new(cx, Vec::<Todo>::new());
		let text_input = Signal::new(cx, "".to_string());
		Container::new(move |_| 
			Flex::column((
				Label::new("TODO app"),
				TextBox::new()
					.on_input(move |cx, value| text_input.set(cx, value.to_string())),
				Flex::column(for_each_keyed(items, |todo| todo.index, |_, todo| todo_view(todo)))
			))
		).style(|s| s.width(Length::Vw(100.0)).height(Length::Vh(100.0)))
	});
	app.run();
}