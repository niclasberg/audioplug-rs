use audioplug::app::SignalSet;
use audioplug::core::{Color, Size, Alignment, Rectangle, Point};
use audioplug::view::{Button, Checkbox, Column, Fill, Label, Row, Slider, TextBox, View};
use audioplug::{window::*, App};

fn main() {
    //let device = Device::new()?;
    //println!("name: {}, id: {}", device.name()?, device.id()?);

    let mut app = App::new();
    let _ = Window::open(&mut app, |ctx| {  
        let checkbox_enabled = ctx.create_signal(false);
        let text = ctx.create_signal("".to_string());
        
        Column::new((
            Rectangle::new(Point::ZERO, Size::new(50.0, 10.0)).fill(Color::BLUE),
            Rectangle::new(Point::ZERO, Size::new(50.0, 10.0)).fill(Color::RED),
            Label::new(text.clone()),
			Row::new((
				Label::new("Slider"),
				Slider::new()
                    .with_range(1.0, 10.0)
                    .with_style(|style| {
                        style.align_self = Some(taffy::AlignItems::Center);
                    })
			)).with_spacing(5.0),
            Row::new((
				Label::new("Checkbox"),
				Checkbox::new(checkbox_enabled.clone())
			)).with_spacing(5.0),
            Row::new((
                Label::new("Button"),
                Button::new(Label::new("Filled"))
                    .on_click(move |ctx| checkbox_enabled.set(ctx, true))
            )).with_spacing(5.0),
            Row::new((
                Label::new("Text input").with_color(Color::BLUE),
                TextBox::new()
                    .on_input(move |cx, str| text.set(cx, str.to_string()))
            )).with_spacing(5.0)
        )).with_alignment(Alignment::Leading)
        .with_spacing(5.0)
    });

    app.run();
}