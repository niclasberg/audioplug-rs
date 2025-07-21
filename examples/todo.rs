use std::sync::atomic::AtomicUsize;

use audioplug::{
    App, KeyEvent,
    core::{Color, Key},
    ui::style::{Length, UiRect},
    ui::*,
    views::*,
};
use rand::prelude::*;
use taffy::AlignSelf;

#[derive(Clone, Copy)]
struct Todo {
    index: usize,
    name: Var<String>,
    completed: Var<bool>,
}

static NEXT_ID: AtomicUsize = AtomicUsize::new(0);
impl Todo {
    pub fn new(cx: &mut dyn CreateContext, name: &str, completed: bool) -> Self {
        Self {
            index: NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            name: Var::new(cx, name.to_string()),
            completed: Var::new(cx, completed),
        }
    }
}

fn main() {
    let mut app = App::new();
    let _ = Window::open(
        &mut app,
        Stateful::new(|cx| {
            let todos = Var::new_with(cx, |cx| vec![Todo::new(cx, "Item1", false)]);
            let text_input = Var::new(cx, "".to_string());

            Container::new(Column::new((
                Label::new("TODO app"),
                TextBox::new(move |cx, value| text_input.set(cx, value.to_string()))
                    .value(text_input)
                    .on_key_event(move |cx, event| {
                        if let KeyEvent::KeyDown {
                            key: Key::Enter, ..
                        } = event
                        {
                            let title = text_input.get_untracked(cx);
                            if !title.is_empty() {
                                todos.update(cx, move |cx, todos| {
                                    todos.push(Todo::new(cx, &title, false))
                                });
                                text_input.set(cx, "".to_string());
                            }

                            return EventStatus::Handled;
                        }
                        EventStatus::Ignored
                    }),
                Row::new((
                    Button::new_with_label("Shuffle").on_click(move |cx| {
                        todos.update(cx, move |_, items| {
                            items.shuffle(&mut rand::rng());
                        });
                    }),
                    Button::new_with_label("Sort").on_click(move |cx| {
                        todos.update(cx, move |cx, items| {
                            items.sort_by_key(|item| item.name.get_untracked(cx));
                        });
                    }),
                )),
                Column::new(todos.map_to_views_keyed(
                    |todo| todo.index,
                    move |todo| {
                        let index = todo.index;
                        todo_view(todo, {
                            move |cx| {
                                todos.update(cx, |_, items| {
                                    items.retain(|x| x.index != index);
                                });
                            }
                        })
                    },
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

fn todo_view<F: Fn(&mut dyn WriteContext) + 'static>(
    todo: &Todo,
    on_remove: F,
) -> impl View + use<F> {
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
