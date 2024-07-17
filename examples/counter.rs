use audioplug::{app::SignalGet, view::{Column, Label}, window::Window, App};



fn main() {
    let mut app = App::new();
    let _ = Window::open(&mut app, |ctx| {  
        let count = ctx.create_signal(0);
        let text = count.clone().map(|cnt| format!("Count is {}", cnt));
        Column::new((
            Label::new("hello"),
        ))
    });
}