use crate::{View, Event, MouseEvent, event::MouseButton, LayoutContext, BuildContext, RenderContext, core::{Size, Constraint, Vector, Color}, Id, LayoutHint};

use super::Label;

pub enum ButtonMessage {
    Clicked
}

pub struct Button {
    label: Label,
    is_hot: bool,
    mouse_down: bool
}

impl Button {
    pub fn new(label: Label) -> Self {
        Self { label, is_hot: false, mouse_down: false }
    }
}

impl View for Button {
	type Message = ButtonMessage;
    type State = <Label as View>::State;

    fn build(&mut self, ctx: &mut BuildContext) -> Self::State {
        ctx.set_number_of_children(1);
        ctx.with_child(Id(0), |ctx| self.label.build(ctx))
    }

    fn rebuild(&mut self, state: &mut Self::State, ctx: &mut BuildContext) {
        ctx.with_child(Id(0), |ctx| self.label.rebuild(state, ctx))
    }

    fn event(&mut self, _state: &mut Self::State, event: crate::Event, ctx: &mut crate::EventContext<ButtonMessage>) {
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
                            ctx.publish_message(ButtonMessage::Clicked);
                            ctx.request_render();
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

    fn layout(&self, state: &mut Self::State, constraint: Constraint, ctx: &mut LayoutContext) -> Size {
        let margin = Size::new(8.0, 8.0);
        let child_constraint = constraint.shrink(margin);
        let label_size = ctx.with_child(Id(0), |ctx| self.label.layout(state, child_constraint, ctx));
        ctx.node.children[0].set_size(label_size);
        ctx.node.children[0].set_offset(Vector::new(4.0, 4.0));
        label_size + margin
    }

    fn render(&self, state: &Self::State, ctx: &mut RenderContext) {
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
        let size = ctx.local_bounds().size();

        ctx.fill(
            &crate::Shape::RoundedRect { size, corner_radius: Size::new(4.0, 4.0) }, 
            ctx.local_bounds().center(), 
            color);
        ctx.with_child(Id(0), |ctx| self.label.render(state, ctx));
    }

    fn layout_hint(&self, _state: &Self::State) -> (crate::LayoutHint, crate::LayoutHint) {
        (LayoutHint::Flexible, LayoutHint::Flexible)
    }
}