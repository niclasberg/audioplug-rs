use crate::{app::{AppState, BuildContext, CallbackContext, EventContext, EventStatus, MouseEventContext, RenderContext, StatusChange, Widget}, core::{Color, Rectangle, RoundedRectangle, Shape, Size}, event::{KeyEvent, MouseButton}, keyboard::Key, style::{DisplayStyle, FlexStyle, Length, Style, UiRect}, MouseEvent};

use super::View;

pub struct Button<V> {
    child: V,
    click_fn: Option<Box<dyn Fn(&mut CallbackContext)>>
}

impl<V: View> Button<V> {
    pub fn new(child: V) -> Self {
        Self { child, click_fn: None }
    }

    pub fn on_click(mut self, f: impl Fn(&mut CallbackContext) + 'static) -> Self {
        self.click_fn = Some(Box::new(f));
        self
    }
}

impl<V: View> View for Button<V> {
    type Element = ButtonWidget; 

    fn build(self, ctx: &mut BuildContext<Self::Element>) -> Self::Element {
        ctx.set_focusable(true);
        ctx.add_child(self.child);
		ctx.set_style(Style {
            padding: UiRect::all(Length::Px(4.0)),
            ..Default::default()
        });

        let widget = ButtonWidget {
            is_hot: false,
            mouse_down: false,
            click_fn: self.click_fn,
        };

        widget
    }
}

const FLEX_STYLE: FlexStyle = FlexStyle::DEFAULT;

pub struct ButtonWidget {
    is_hot: bool,
    mouse_down: bool,
    click_fn: Option<Box<dyn Fn(&mut CallbackContext)>>
}

impl Widget for ButtonWidget {
	fn debug_label(&self) -> &'static str {
		"Button"
	}

    fn mouse_event(&mut self, event: MouseEvent, ctx: &mut MouseEventContext) -> EventStatus {
        match event {
            MouseEvent::Down { button, position, .. } | MouseEvent::DoubleClick { button, position, .. } if ctx.bounds().contains(position) => {
                if button == MouseButton::LEFT {
                    ctx.capture_mouse();
                    ctx.request_render();
                }
                EventStatus::Handled
            },
            MouseEvent::Up { button, position, .. } if button == MouseButton::LEFT => {
                ctx.release_capture();
				if ctx.bounds().contains(position) {
					if let Some(f) = self.click_fn.as_ref() {
						f(&mut ctx.as_callback_context());
					}
				}
				ctx.request_render();
                EventStatus::Handled
            },
            _ => ctx.forward_to_children(event)
        }
    }

    fn status_updated(&mut self, event: StatusChange, ctx: &mut EventContext) {
        match event {
            StatusChange::FocusGained => {
                self.is_hot = true;
                ctx.request_render();
            },
            StatusChange::FocusLost => {
                self.is_hot = false;
                ctx.request_render();
            },
            StatusChange::MouseCaptureLost => {
                if self.mouse_down {
                    self.mouse_down = false;
                    ctx.request_render();
                }
            },
            _ => {}
        }
    }
      
	fn key_event(&mut self, event: KeyEvent, ctx: &mut EventContext) -> EventStatus {
		match event {
			KeyEvent::KeyDown { key, ..} if key == Key::Enter => {
				if let Some(f) = self.click_fn.as_ref() {
					f(&mut ctx.as_callback_context());
				} 
				EventStatus::Handled
			},
			_ => EventStatus::Ignored
		}
	}

    fn render(&mut self, ctx: &mut RenderContext) {
        let color = if ctx.has_mouse_capture() {
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

        let shape = RoundedRectangle::new(ctx.global_bounds(), Size::new(4.0, 4.0));
        ctx.fill(shape, color);
        ctx.stroke(shape, Color::BLACK, 1.0);
        
        ctx.render_children()
    }

    fn display_style(&self) -> DisplayStyle {
        DisplayStyle::Flex(&FLEX_STYLE)
    }
}