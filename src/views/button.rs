use crate::{core::{Color, Constraint, Shape, Size, Vector}, event::MouseButton, BuildContext, Event, Id, LayoutContext, LayoutHint, MouseEvent, RenderContext, View, Widget};

pub struct Button<F, W> {
    child: W,
    is_hot: bool,
    mouse_down: bool,
    click_fn: Option<F>
}

impl<F: Fn(), W: Widget> Button<F, W> {
    pub fn new(child: W) -> Self {
        Self { child, is_hot: false, mouse_down: false, click_fn: None }
    }

    pub fn on_click(mut self, f: F) -> Self {
        self.click_fn = Some(f);
        self
    }
}

impl<F: Fn(), W: Widget> Widget for Button<F, W> {
    /*  build(&mut self, ctx: &mut BuildContext) -> Self::State {
        ctx.set_number_of_children(1);
        ctx.with_child(Id(0), |ctx| self.label.build(ctx))
    }*/

    /*fn rebuild(&mut self, state: &mut Self::State, ctx: &mut BuildContext) {
        ctx.with_child(Id(0), |ctx| self.label.rebuild(state, ctx))
    }*/

    fn event(&mut self, event: crate::Event, ctx: &mut crate::EventContext<()>) {
        match event {
            Event::Mouse(mouse_event) => match mouse_event {
                MouseEvent::Down { button, position } if ctx.local_bounds().contains(position) => {
                    ctx.set_handled();
                    if button == MouseButton::LEFT {
                        self.mouse_down = true;
                        ctx.request_render();
                    }
                },
                MouseEvent::Up { button, position } if button == MouseButton::LEFT => {
                    if self.mouse_down {
                        self.mouse_down = false;
                        ctx.set_handled();
                        if ctx.local_bounds().contains(position) {
                            ctx.request_render();
                            if let Some(f) = self.click_fn.as_ref() {
                                f();
                            }
                        }
                    }
                },
                MouseEvent::Enter  => {
                    self.is_hot = true;
                    ctx.request_render();
                },
                MouseEvent::Exit  => {
                    self.is_hot = false;
                    ctx.request_render();
                }
                _ => {}
            },
            _ => {}
        }
    }

    fn layout(&mut self, constraint: Constraint, ctx: &mut LayoutContext) -> Size {
        let margin = Size::new(8.0, 8.0);
        let child_constraint = constraint.shrink(margin);
        let label_size = ctx.with_child(Id(0), |ctx| self.child.layout(child_constraint, ctx));
        ctx.node.children[0].set_size(label_size);
        ctx.node.children[0].set_offset(Vector::new(4.0, 4.0));
        label_size + margin
    }

    fn render(&mut self, ctx: &mut RenderContext) {
        let color = if self.mouse_down {
            if self.is_hot { 
                Color::from_rgb8(0, 66, 37)
            } else {
                Color::from_rgb8(106, 156, 137)
            }
        } else {
            if self.is_hot { 
                Color::from_rgb8(121, 153, 141)
            } else { 
                Color::from_rgb8(106, 156, 137)
            }
        };

        let shape = Shape::RoundedRect { rect: ctx.local_bounds(), corner_radius: Size::new(4.0, 4.0) };
        ctx.fill(shape, color);
        ctx.stroke(shape, Color::BLACK, 1.0);
        ctx.with_child(Id(0), |ctx| self.child.render(ctx));
    }

    fn layout_hint(&self) -> (crate::LayoutHint, crate::LayoutHint) {
        (LayoutHint::Flexible, LayoutHint::Flexible)
    }
}