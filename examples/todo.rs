use std::sync::atomic::AtomicUsize;

use audioplug::{
    app::*,
    core::{Color, Key},
    style::{Length, UiRect},
    views::*,
    App, KeyEvent,
};
use taffy::AlignSelf;

#[derive(Clone, Copy)]
struct Todo {
    index: usize,
    name: Signal<String>,
    completed: Signal<bool>,
}

static NEXT_ID: AtomicUsize = AtomicUsize::new(0);
impl Todo {
    pub fn new(cx: &mut dyn CreateContext, name: &str, completed: bool) -> Self {
        Self {
            index: NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            name: Signal::new(cx, name.to_string()),
            completed: Signal::new(cx, completed),
        }
    }
}

fn main() {
    let mut app = App::new();
    let _ = Window::open(
        &mut app,
        Stateful::new(|cx| {
            let items = Signal::new_with(cx, |cx| vec![Todo::new(cx, "Item1", false)]);
            let mut last_id = 1;
            let tests = Signal::new(cx, vec![1]);
            let text_input = Signal::new(cx, "".to_string());
            Container::new(Column::new((
                Label::new("TODO app"),
                TextBox::new()
                    .on_input(move |cx, value| text_input.set(cx, value.to_string()))
                    .on_key_event(move |cx, event| {
                        if let KeyEvent::KeyDown { key, .. } = event {
                            if Key::Enter == key {
                                let title = text_input.get_untracked(cx);
                                if !title.is_empty() {
                                    last_id += 1;
                                    items.update(cx, move |cx, todos| {
                                        todos.push(Todo::new(cx, &title, false))
                                    });
                                    tests.update(cx, move |_, t| t.push(last_id));
                                    text_input.set(cx, "".to_string());
                                }

                                return EventStatus::Handled;
                            }
                        }
                        EventStatus::Ignored
                    }),
                Column::new(items.map_to_views_keyed(
                    |todo| todo.index,
                    move |todo| {
                        let index = todo.index;
                        todo_view(todo, {
                            move |cx| {
                                items.update(cx, |_, items| {
                                    items.retain(|x| x.index != index);
                                });
                                tests.update(cx, |_, items| {
                                    items.pop();
                                });
                            }
                        })
                    },
                )),
                Column::new(view_for_each(
                    move |cx| tests.get(cx),
                    |i| Label::new(format!("AA: {}", i)),
                )),
            )))
            .style(|s| {
                s.width(Length::Vw(100.0))
                    .height(Length::Vh(100.0))
                    .background(Color::WHEAT)
            })
        }),
    );
    app.run();
}

fn todo_view(todo: &Todo, on_remove: impl Fn(&mut dyn WriteContext) + 'static) -> impl View {
    let completed = todo.completed;
    Row::new((
        Checkbox::new()
            .checked(todo.completed)
            .on_click(move |cx| completed.update(cx, |_, value| *value = !*value)),
        Label::new(todo.name).style(|style| style.flex_grow(1.0)),
        Button::new(Label::new("Remove"))
            .on_click(move |cx| on_remove(cx))
            .style(|style| style.justify_self(AlignSelf::End)),
    ))
    .v_align_center()
    .spacing(Length::Px(5.0))
    .style(|s| {
        s.background(
            todo.completed
                .map(|&c| if c { Color::GREEN } else { Color::RED }.into()),
        )
        .padding(UiRect::all_px(5.0))
        .width(Length::Percent(100.0))
    })
}
