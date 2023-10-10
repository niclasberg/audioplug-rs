use crate::{View, Event, MouseEvent, event::MouseButton, LayoutContext, BuildContext, RenderContext, core::{Size, Constraint, Vector, Color, Point}, Id};

use super::Label;

pub enum ButtonMessage {
    Clicked
}

pub struct Button {
    label: Label
}

impl Button {
    pub fn new(label: Label) -> Self {
        Self { label }
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

    fn event(&mut self, state: &mut Self::State, event: crate::Event, ctx: &mut crate::EventContext<ButtonMessage>) {
        match event {
            Event::Mouse(mouse_event) => match mouse_event {
                MouseEvent::Down { button, .. } if button == MouseButton::LEFT => {
                    ctx.publish_message(ButtonMessage::Clicked);
                    ctx.request_render();
                },
                MouseEvent::Up { button, .. } if button == MouseButton::LEFT => {
                    ctx.request_render();
                }
                _ => {}
            },
            _ => {}
        }
    }

    fn layout(&self, state: &mut Self::State, constraint: Constraint, ctx: &mut LayoutContext) -> Size {
        let margin = Size::new(4.0, 4.0);
        let child_constraint = constraint.shrink(margin);
        let label_size = ctx.with_child(Id(0), |ctx| self.label.layout(state, child_constraint, ctx));
        ctx.node.children[0].set_offset(Vector::new(2.0, 2.0));
        label_size + margin
    }

    fn render(&self, state: &Self::State, ctx: &mut RenderContext) {
        ctx.fill(&crate::Shape::RoundedRect { size: ctx.local_bounds().size(), corner_radius: Size::new(2.0, 2.0) }, Point::ZERO, Color::GREEN);
        ctx.with_child(Id(0), |ctx| self.label.render(state, ctx));
    }
}