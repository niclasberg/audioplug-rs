use crate::{app::{AppState, BuildContext, EventContext, LayoutContext, RenderContext}, core::{Color, Shape, Size}, event::{KeyEvent, MouseButton}, keyboard::Key, MouseEvent};
use super::{EventStatus, View, Widget};

pub struct Button<V> {
    child: V,
    click_fn: Option<Box<dyn Fn(&mut AppState)>>
}

impl<V: View> Button<V> {
    pub fn new(child: V) -> Self {
        Self { child, click_fn: None }
    }

    pub fn on_click(mut self, f: impl Fn(&mut AppState) + 'static) -> Self {
        self.click_fn = Some(Box::new(f));
        self
    }
}

impl<V: View> View for Button<V> {
    type Element = ButtonWidget; 

    fn build(self, ctx: &mut BuildContext) -> Self::Element {
        ctx.set_focusable(true);
        ctx.add_child(self.child);

        let widget = ButtonWidget {
            is_hot: false,
            mouse_down: false,
            click_fn: self.click_fn,
        };

        widget
    }
}

pub struct ButtonWidget {
    is_hot: bool,
    mouse_down: bool,
    click_fn: Option<Box<dyn Fn(&mut AppState)>>
}

impl Widget for ButtonWidget {
    fn mouse_event(&mut self, event: MouseEvent, ctx: &mut EventContext) -> EventStatus {
        match event {
            MouseEvent::Down { button, position } if ctx.bounds().contains(position) => {
                if button == MouseButton::LEFT {
                    println!("Down");
                    self.mouse_down = true;
                    ctx.capture_mouse();
                    ctx.request_render();
                }
                EventStatus::Handled
            },
            MouseEvent::Up { button, position } if button == MouseButton::LEFT => {
                if self.mouse_down {
                    self.mouse_down = false;
                    if ctx.bounds().contains(position) {
                        ctx.request_render();
                        if let Some(f) = self.click_fn.as_ref() {
                            f(ctx.app_state_mut());
                        }
                    }
                }
                EventStatus::Handled
            },
            _ => EventStatus::Ignored
        }
    }

    fn mouse_enter_exit(&mut self, has_mouse_over: bool, ctx: &mut EventContext)  -> EventStatus {
        self.is_hot = has_mouse_over;
        ctx.request_render();
        EventStatus::Handled
    }

	fn key_event(&mut self, event: KeyEvent, ctx: &mut EventContext) -> EventStatus {
		match event {
			KeyEvent::KeyDown { key, ..} if key == Key::Enter => {
				if let Some(f) = self.click_fn.as_ref() {
					f(ctx.app_state_mut());
				} 
				EventStatus::Handled
			},
			_ => EventStatus::Ignored
		}
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
        
        //self.child.render(ctx);
    }
    
    fn style(&self) -> taffy::Style {
        let padding = taffy::LengthPercentage::Length(4.0);
        taffy::Style {
            padding: taffy::Rect { left: padding, right: padding, top: padding, bottom: padding },
            justify_content: Some(taffy::JustifyContent::Center),
            ..Default::default()
        }
    }
}