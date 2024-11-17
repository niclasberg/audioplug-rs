use std::path::Path;
use audioplug::app::{Effect, Memo, Signal, SignalGet, SignalSet, Window};
use audioplug::core::{Alignment, Border, Color};
use audioplug::view::{Button, Checkbox, Column, Image, Label, Row, Slider, TextBox, View};
use audioplug::App;

fn main() {
    //let device = Device::new()?;
    //println!("name: {}, id: {}", device.name()?, device.id()?);

    let mut app = App::new();
    let _ = Window::open(&mut app, |cx| {  
        let checkbox_enabled = Signal::new(cx, false);
        let text = Signal::new(cx, "".to_string());
        let slider_value = Signal::new(cx, 100.0);

        Effect::new_with_state(cx, move |cx, cnt| {
            let cnt = cnt.unwrap_or(0);
            println!("Cnt: {}, Slider value: {}, enabled: {}", cnt, slider_value.get(cx), checkbox_enabled.get(cx));
            cnt + 1
        });

        Column::new((
            Label::new(text.clone()).border(Border { color: Color::GREEN, width: 2.0 }),
			Row::new((
				Label::new("Slider"),
				Slider::new()
                    .range(1.0, 500.0)
					.value(slider_value)
                    .on_value_changed(move |cx, value| slider_value.set(cx, value))
			)).spacing(5.0),
            Row::new((
				Label::new("Checkbox"),
				Checkbox::new(checkbox_enabled.clone())
			)).spacing(5.0),
            Row::new((
                Label::new("Button"),
                Button::new(Label::new("Filled"))
                    .on_click(move |ctx| {
                        let current = checkbox_enabled.get(ctx);
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
                Label::new("Text input").color(Color::BLUE),
                TextBox::new()
                    .on_input(move |cx, str| text.set(cx, str.to_string()))
            )).spacing(5.0)
        )).align(Alignment::Leading)
        .spacing(5.0)
    });

    app.run();
}