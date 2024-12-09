use audioplug::{app::{Signal, SignalGet, SignalSet, Window}, core::{Color, Size}, style::{Length, UiRect}, view::{Button, Container, Flex, IndexedViewSeq, Label, View}, App};

fn main() {
    let mut app = App::new();
    let _ = Window::open(&mut app, |cx| {  
        let count = Signal::new(cx, 0);
		Container::new(move |_|
			Flex::column((
				Label::new(count.map(|cnt| format!("Count: {}", cnt))),
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
				Label::new("No children to show")
					.style(|style| style.hidden(count.map(|x| *x > 0))),
				Flex::column(IndexedViewSeq::new(count.map(|&x| x.max(0) as usize), |_, i| {
					Label::new(format!("Child {}", i+1))
				}))
			))
			.spacing(Length::Px(10.0))
			.style(|style| style
				.width(Length::Percent(30.0))
				.min_width(Length::Px(200.0))
				.padding(UiRect::all_px(15.0))
				.corner_radius(Size::new(10.0, 10.0))
				.background(count.map(|cnt| 
					if *cnt >= 0 {
						Color::from_rgb(0.8, 0.8, 0.8)
					} else {
						Color::RED
					}))
			)
		).style(|style| style
			.height(Length::Vh(100.0))
			.width(Length::Vw(100.0)))
    });
	app.run();
}