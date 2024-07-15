use audioplug::core::{Color, Size, Alignment, Shape, Rectangle, Point};
use audioplug::view::{Button, Checkbox, Column, Fill, Label, Row, Scroll, Slider, TextBox, View};
use audioplug::{window::*, App};

fn main() {
    //let device = Device::new()?;
    //println!("name: {}, id: {}", device.name()?, device.id()?);

    let mut app = App::new();
    let _ = Window::open(&mut app, |ctx| {  
        let checkbox_enabled = ctx.create_signal(true);

        Column::new((
            Rectangle::new(Point::ZERO, Size::new(50.0, 10.0)).fill(Color::BLUE),
            Rectangle::new(Point::ZERO, Size::new(50.0, 10.0)).fill(Color::RED),
			Row::new((
				Label::new("Slider"),
				Slider::new().with_range(1.0, 10.0).with_style(|style| {
					style.align_self = Some(taffy::AlignItems::Center);
				})
			)).with_spacing(5.0),
            Row::new((
				Label::new("Checkbox"),
				Checkbox::new(checkbox_enabled)
			)).with_spacing(5.0),
            Row::new((
                Label::new("Button").with_style(|style| {
                    style.size = taffy::Size::from_lengths(300.0, 40.0);
                }),
                Button::new(Label::new("Filled")).on_click(|| println!("Clicked"))
            )).with_spacing(5.0),
            Row::new((
                Label::new("Text input").with_color(Color::BLUE),
                TextBox::new()
            )).with_spacing(5.0)
        )).with_alignment(Alignment::Leading)
        .with_spacing(5.0)
    });

    app.run();
}