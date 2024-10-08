use std::path::Path;

use audioplug::app::{SignalGet, SignalSet, Window};
use audioplug::core::{Color, Size, Alignment, Rectangle, Point};
use audioplug::view::{Button, Checkbox, Column, Fill, Image, Label, Row, Slider, TextBox, View};
use audioplug::App;

fn main() {
    //let device = Device::new()?;
    //println!("name: {}, id: {}", device.name()?, device.id()?);

    let mut app = App::new();
    let _ = Window::open(&mut app, |ctx| {  
        let checkbox_enabled = ctx.create_signal(false);
        let text = ctx.create_signal("".to_string());
        let slider_value = ctx.create_signal(100.0);
        
        Column::new((
            Label::new(text.clone()),
			Row::new((
				Label::new("Slider"),
				Slider::new()
                    .range(1.0, 500.0)
					.value(slider_value)
                    .on_value_changed(move |ctx, value| slider_value.set(ctx, value))
			)).spacing(5.0),
            Row::new((
				Label::new("Checkbox"),
				Checkbox::new(checkbox_enabled.clone())
			)).spacing(5.0),
            Row::new((
                Label::new("Button"),
                Button::new(Label::new("Filled"))
                    .on_click(move |ctx| {
                        let current = checkbox_enabled.get_untracked(ctx);
                        checkbox_enabled.set(ctx, !current)
                    })
            )).spacing(5.0),
            Row::new((
                Label::new("Image"),
                Image::from_file(Path::new("/Users/niklas.berg/Desktop/Screenshot 2024-04-24 at 09.49.30.png"))
					.max_width(200.0)
                    .height(slider_value)
            )),
            Row::new((
                Label::new("Text input").with_color(Color::BLUE),
                TextBox::new()
                    .on_input(move |cx, str| text.set(cx, str.to_string()))
            )).spacing(5.0)
        )).align(Alignment::Leading)
        .spacing(5.0)
    });

    app.run();
}