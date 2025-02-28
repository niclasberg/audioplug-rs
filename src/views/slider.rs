use crate::{app::{Accessor, AppState, BuildContext, EventContext, EventStatus, LinearGradient, MouseEventContext, ParamEditor, ParamSignal, RenderContext, StatusChange, Widget}, core::{Color, Point, Rectangle, RoundedRectangle, Shape, Size, UnitPoint}, event::MouseButton, keyboard::Key, param::{AnyParameter, NormalizedValue, PlainValue}, style::{DisplayStyle, Length, Measure, Style}, KeyEvent, MouseEvent};

use super::View;

pub struct Slider {
    min: f64,
    max: f64,
	value: Option<Accessor<f64>>,
    on_drag_start: Option<Box<dyn Fn(&mut AppState)>>, 
    on_drag_end: Option<Box<dyn Fn(&mut AppState)>>, 
    on_value_changed: Option<Box<dyn Fn(&mut AppState, f64)>>,
}

impl Slider {
    pub fn new() -> Self {
        Self { 
            min: 0.0, 
            max: 1.0, 
			value: None,
            on_drag_start: None,
            on_drag_end: None,
            on_value_changed: None 
        }
    }

    pub fn on_value_changed(mut self, f: impl Fn(&mut AppState, f64) + 'static) -> Self {
        self.on_value_changed = Some(Box::new(f));
        self
    }

	pub fn on_drag_start(mut self, f: impl Fn(&mut AppState) + 'static) -> Self {
		self.on_drag_start = Some(Box::new(f));
		self
	}

	pub fn on_drag_end(mut self, f: impl Fn(&mut AppState) + 'static) -> Self {
		self.on_drag_end = Some(Box::new(f));
		self
	}

	pub fn value(mut self, value: impl Into<Accessor<f64>>) -> Self {
		self.value = Some(value.into());
		self
	}

    pub fn range(mut self, min: f64, max: f64) -> Self {
        self.min = min;
        self.max = max;
        self
    }
}

impl View for Slider {
    type Element = SliderWidget;

    fn build(self, ctx: &mut BuildContext<Self::Element>) -> Self::Element {
        ctx.set_focusable(true);
		ctx.set_style(Style {
            size: Size::new(Length::Auto, Length::Px(10.0)),
            ..Default::default()
        });

		let position_normalized = if let Some(value) = self.value {
			let position = value.get_and_bind(ctx, move |value, mut widget| {
				widget.position_normalized = normalize_value(widget.min, widget.max, value);
				widget.request_render();
			});
			normalize_value(self.min, self.max, position)
		} else {
			0.0
		};

        SliderWidget {
            position_normalized,
            state: State::Idle,
            min: self.min,
            max: self.max, 
            on_drag_start: self.on_drag_start,
            on_drag_end: self.on_drag_end,
            on_value_changed: self.on_value_changed
        }
    }
}

fn normalize_value(min: f64, max: f64, value: f64) -> f64 {
	((value - min) / (max - min)).clamp(0.0, 1.0)
}

pub struct ParameterSlider<P: AnyParameter> {
    editor: ParamEditor<P>,
	signal: Accessor<NormalizedValue>
}

impl<P: AnyParameter> ParameterSlider<P> {
    pub fn new(parameter: &P) -> Self {
		let signal = parameter.as_signal_normalized().into();
        let editor = ParamEditor::new(parameter);
        Self {
            editor,
			signal
        }
    }
}

impl<P: AnyParameter> View for ParameterSlider<P> {
    type Element = SliderWidget;

    fn build(self, ctx: &mut BuildContext<Self::Element>) -> Self::Element {
        let editor = self.editor;
        let slider = Slider::new()
            .range(editor.info(ctx).min_value().into(), editor.info(ctx).max_value().into())
			.on_drag_start(move |cx| {
				editor.begin_edit(cx);
			})
			.on_drag_end(move |cx| {
				editor.end_edit(cx);
			})
            .on_value_changed(move |cx, value| {
                editor.set_value_plain(cx, PlainValue::new(value));
            });

		let normalized_position = self.signal.get_and_bind(ctx, |value, mut widget| {
			widget.position_normalized = value.0;
			widget.request_render();
		});

        let mut widget = ctx.build(slider);
		widget.position_normalized = normalized_position.into();
		widget
    }
}

pub struct SliderWidget {
    /// Normalized position, between 0 and 1
    position_normalized: f64,
    state: State,
    min: f64,
    max: f64,
    on_drag_start: Option<Box<dyn Fn(&mut AppState)>>, 
    on_drag_end: Option<Box<dyn Fn(&mut AppState)>>, 
    on_value_changed: Option<Box<dyn Fn(&mut AppState, f64)>>
}

#[derive(Debug, PartialEq)]
enum State {
    Idle,
    KnobHover,
    Dragging
}

impl SliderWidget {
    fn slider_position(&self, bounds: Rectangle) -> Point {
        let slider_bounds = self.inner_bounds(bounds);
        let knob_x = slider_bounds.left() + self.position_normalized * slider_bounds.width();
        let knob_y = slider_bounds.center().y;
        Point::new(knob_x, knob_y)
    }

    fn inner_bounds(&self, bounds: Rectangle) -> Rectangle {
        bounds.shrink_x(5.0)
    }

    fn knob_shape(&self, bounds: Rectangle) -> Shape {
        Shape::circle(self.slider_position(bounds), 5.0)
    }

    fn absolute_to_normalized_position(position: Point, bounds: Rectangle) -> f64 {
        ((position.x - bounds.left() - 2.5) / (bounds.width() - 5.0)).clamp(0.0, 1.0)
    }

    fn set_position(&mut self, app_state: &mut AppState, normalized_position: f64) -> bool {
        if normalized_position != self.position_normalized {
            self.position_normalized = normalized_position;
            if let Some(f) = self.on_value_changed.as_ref() {
                f(app_state, self.min + (self.max - self.min) * self.position_normalized);
            }
            true
        } else {
            false
        }
    }
}

impl Measure for SliderWidget {
    fn measure(&self, 
        _style: &Style,
        width: Option<f64>, 
        height: Option<f64>, 
        available_width: taffy::AvailableSpace, 
        _available_height: taffy::AvailableSpace) -> Size<f64> 
    {
        let width = width.unwrap_or(match available_width {
            taffy::AvailableSpace::Definite(x) => x.into(),
            taffy::AvailableSpace::MinContent => 5.0,
            taffy::AvailableSpace::MaxContent => 100.0,
        });
        let height = height.unwrap_or(5.0);

        Size::new(width, height)
    }
}

impl Widget for SliderWidget {
	fn debug_label(&self) -> &'static str {
		"Slider"
	}

    fn mouse_event(&mut self, event: MouseEvent, ctx: &mut MouseEventContext) -> EventStatus {
        match event {
            MouseEvent::Down { button, position } => {
				if button == MouseButton::LEFT && self.state != State::Dragging {
                	if !self.knob_shape(ctx.bounds()).hit_test(position) {
                        let normalized_position = Self::absolute_to_normalized_position(position, ctx.bounds());
						if self.set_position(ctx.app_state_mut(), normalized_position) {
							ctx.request_render();
						}
					}
					ctx.capture_mouse();
					ctx.request_render();
					if let Some(f) = self.on_drag_start.as_ref() {
						f(ctx.app_state_mut());
					}
					self.state = State::Dragging;
				}
                EventStatus::Handled
            },
            MouseEvent::Moved { position, .. } => {
                match self.state {
                    State::Idle => {
                        if self.knob_shape(ctx.bounds()).hit_test(position) {
                            ctx.request_render();
                            self.state = State::KnobHover;
                        }
                    },
                    State::KnobHover => {
                        if !self.knob_shape(ctx.bounds()).hit_test(position) {
                            ctx.request_render();
                            self.state = State::Idle;
                        }
                    },
                    State::Dragging => {
                        let normalized_position = Self::absolute_to_normalized_position(position, ctx.bounds());
                        if self.set_position(ctx.app_state_mut(), normalized_position) {
                            ctx.request_render();
                        }
                    },
                }
                EventStatus::Handled
            },
            MouseEvent::Up { button, position } => {
                if button == MouseButton::LEFT {
                    if self.state == State::Dragging {
                        if let Some(f) = self.on_drag_end.as_ref() {
                            f(ctx.app_state_mut())
                        }
                    }
                    ctx.release_capture();
                    ctx.request_render();
                    self.state = if self.knob_shape(ctx.bounds()).hit_test(position) {
                        State::KnobHover
                    } else {
                        State::Idle
                    };
                }
                EventStatus::Handled
            }
            _ => EventStatus::Ignored
        }
    }

    fn key_event(&mut self, event: KeyEvent, ctx: &mut EventContext) -> EventStatus {
        match event {
            crate::KeyEvent::KeyDown { key, .. } => {
                match key {
                    Key::Left => {
                        let new_position = (self.position_normalized - 0.1).clamp(0.0, 1.0);
                        if self.set_position(ctx.app_state_mut(), new_position) {
                            ctx.request_render();
                        }
                        EventStatus::Handled
                    },
                    Key::Right => {
                        let new_position = (self.position_normalized + 0.1).clamp(0.0, 1.0);
                        if self.set_position(ctx.app_state_mut(), new_position) {
                            ctx.request_render();
                        }
                        EventStatus::Handled
                    },
                    _ => EventStatus::Ignored
                }
            },
            _ => EventStatus::Ignored
        }
    }

    fn status_updated(&mut self, event: StatusChange, ctx: &mut EventContext) {
        match event {
            StatusChange::FocusGained | StatusChange::FocusLost => {
                ctx.request_render()
            },
            StatusChange::MouseCaptureLost => {
                if self.state == State::Dragging {
                    self.state = State::Idle;
                    if let Some(f) = self.on_drag_end.as_ref() {
                        f(ctx.app_state_mut())
                    }
                }
            },
            _ => {}
        }
    }

    fn render(&mut self, ctx: &mut RenderContext) {
        let bounds = ctx.content_bounds();
        let center = bounds.center();
        let width = bounds.width();
        let slider_position = self.slider_position(bounds);
        let knob_color = match self.state {
            State::Idle => Color::BLACK,
            State::KnobHover => Color::from_rgb(0.5, 0.5, 0.5),
            State::Dragging => Color::from_rgb(0.75, 0.75, 0.75),
        };

        if ctx.has_focus() {
            //ctx.stroke(bounds, Color::RED, 1.0);
        }

        let background_rect = RoundedRectangle::new(Rectangle::from_center(center, Size::new(width - 2.0, 3.0)), Size::new(1.5, 1.5));
        let range_indicator_rect = Rectangle::from_ltrb(
            bounds.left() + 2.5, 
            center.y - 1.0, 
            slider_position.x, 
            center.y + 1.0);

        let gradient = LinearGradient::new((Color::BLACK.with_alpha(0.2), Color::WHITE.with_alpha(0.2)), UnitPoint::TOP_CENTER, UnitPoint::BOTTOM_CENTER);
        ctx.stroke(background_rect, &gradient, 1.0);
        ctx.fill(background_rect, Color::BLACK.with_alpha(0.3));
        ctx.fill(RoundedRectangle::new(range_indicator_rect, Size::new(1.0, 1.0)), Color::NEON_GREEN);
        ctx.fill(self.knob_shape(bounds), knob_color);
    }

    fn display_style(&self) -> DisplayStyle {
        DisplayStyle::Leaf(self)
    }
}