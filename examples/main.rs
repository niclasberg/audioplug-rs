use std::path::Path;
use audioplug::app::*;
use audioplug::core::{Color, Size};
use audioplug::style::Length;
use audioplug::view::*;
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

        Flex::column((
            Label::new(text)
				.style(|s| s
					.border(Length::Px(2.0), Color::GREEN)
					.corner_radius(Size::new(2.0, 2.0))),
			Flex::row((
				Label::new("Slider"),
				Slider::new()
                    .range(1.0, 500.0)
					.value(slider_value)
                    .on_value_changed(move |cx, value| slider_value.set(cx, value))
			)).spacing(Length::Px(5.0))
            .style(|s| s.background(Color::RED)),
            Flex::row((
				Label::new("Checkbox"),
				Checkbox::new(checkbox_enabled)
			)).spacing(Length::Px(5.0)),
            Flex::row((
                Label::new("Button"),
                Button::new(Label::new("Filled"))
                    .on_click(move |cx| {
						checkbox_enabled.update(cx, |enabled| *enabled = !*enabled );
                    })
            )).spacing(Length::Px(5.0)),
            Flex::row((
                Label::new("Image"),
                Image::from_file(Path::new("/Users/niklas.berg/Desktop/Screenshot 2024-04-24 at 09.49.30.png"))
                    .style(|style| style
                        .max_width(Length::Px(200.0))
                        .height(slider_value.map(Length::from_px))
                    )
            )),
            Flex::row((
                Label::new("Text input").color(Color::BLUE),
                TextBox::new()
                    .on_input(move |cx, str| text.set(cx, str.to_string()))
            )).spacing(Length::Px(5.0))
        )).spacing(Length::Px(5.0))
    });

    app.run();
}