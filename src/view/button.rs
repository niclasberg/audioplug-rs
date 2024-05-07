use crate::{core::{Color, Shape, Size}, event::MouseButton, Event, Id, MouseEvent};
use super::{BuildContext, EventContext, LayoutContext, RenderContext, View, Widget, WidgetNode};

pub struct Button<V> {
    child: V,
    click_fn: Option<Box<dyn Fn()>>
}

impl<V: View> Button<V> {
    pub fn new(child: V) -> Self {
        Self { child, click_fn: None }
    }

    pub fn on_click(mut self, f: impl Fn() + 'static) -> Self {
        self.click_fn = Some(Box::new(f));
        self
    }
}

impl<V: View> View for Button<V> {
    type Element = ButtonWidget; 

    fn build(self, ctx: &mut BuildContext) -> Self::Element {
        ButtonWidget {
            child: ctx.build_child(Id(0), self.child),
            is_hot: false,
            mouse_down: false,
            click_fn: self.click_fn,
        }
    }
}

pub struct ButtonWidget {
    child: WidgetNode,
    is_hot: bool,
    mouse_down: bool,
    click_fn: Option<Box<dyn Fn()>>
}

impl Widget for ButtonWidget {
    fn event(&mut self, event: crate::Event, ctx: &mut EventContext) {
        match event {
            Event::Mouse(mouse_event) => match mouse_event {
                MouseEvent::Down { button, position } if ctx.bounds().contains(position) => {
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
                        if ctx.bounds().contains(position) {
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

    fn layout(&mut self, inputs: taffy::LayoutInput, ctx: &mut LayoutContext) -> taffy::LayoutOutput {
        ctx.compute_block_layout(self, inputs)
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

        let shape = Shape::RoundedRect { rect: ctx.global_bounds(), corner_radius: Size::new(4.0, 4.0) };
        ctx.fill(shape, color);
        ctx.stroke(shape, Color::BLACK, 1.0);
        
        self.child.render(ctx);
    }
    
    fn style(&self) -> taffy::Style {
        let padding = taffy::LengthPercentage::Length(4.0);
        taffy::Style {
            padding: taffy::Rect { left: padding, right: padding, top: padding, bottom: padding },
            justify_content: Some(taffy::JustifyContent::Center),
            ..Default::default()
        }
    }
    
    fn child_count(&self) -> usize { 1 }
    fn get_child<'a>(&'a self, i: usize) -> &'a WidgetNode { 
        assert_eq!(i, 0); 
        &self.child
    }
    
    fn get_child_mut<'a>(&'a mut self, i: usize) -> &'a mut WidgetNode {    
        assert_eq!(i, 0); 
        &mut self.child
    }
}