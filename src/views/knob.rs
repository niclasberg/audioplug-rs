use crate::{app::{Accessor, EventStatus, RenderContext, StatusChange, View, Widget}, core::{Circle, Color, Ellipse, Point, Rectangle, Size}, keyboard::Modifiers, style::{DisplayStyle, Measure}, MouseButton, MouseEvent};

pub struct Knob {
    value: Option<Accessor<f64>>
}

impl Knob {
    pub fn new() -> Self {
        Self { 
            value: None
        }
    }
}

impl View for Knob {
    type Element = KnobWidget;

    fn build(self, ctx: &mut crate::app::BuildContext<Self::Element>) -> Self::Element {
        KnobWidget {
            normalized_value: 0.0,
            last_mouse_pos: None
        }
    }
}

pub struct KnobWidget {
    normalized_value: f64,
    last_mouse_pos: Option<Point>,
}

impl KnobWidget {
    fn shape(&self, bounds: Rectangle) -> Circle {
        let center = bounds.center();
        let radius = bounds.size().width.min(bounds.size().height) / 2.0;
        Circle::new(center, radius)
    }

    fn is_inside_knob(&self, bounds: Rectangle, point: Point) -> bool {
        self.shape(bounds).contains(point)
    }

    fn min_angle(&self) -> f64 {
        -1.25 * std::f64::consts::PI
    }

    fn max_angle(&self) -> f64 {
        0.25 * std::f64::consts::PI
    }

    fn current_angle(&self) -> f64 {
        self.min_angle() * (1.0 - self.normalized_value) + self.max_angle() * self.normalized_value
    }
}

impl Measure for KnobWidget {
    fn measure(
        &self,
        _style: &crate::style::Style,
        width: Option<f64>,
        height: Option<f64>,
        _available_width: taffy::AvailableSpace,
        _available_height: taffy::AvailableSpace,
    ) -> Size {
        match (width, height) {
            (Some(width), Some(height)) => Size::new(width, height),
            (Some(len), None) | (None, Some(len)) => Size::new(len, len),
            (None, None) => Size::new(20.0, 20.0)
        }
    }
}

impl Widget for KnobWidget {
    fn display_style(&self) -> DisplayStyle {
        DisplayStyle::Leaf(self)
    }

    fn debug_label(&self) -> &'static str {
        "Knob"
    }

    fn mouse_event(&mut self, event: MouseEvent, cx: &mut crate::app::MouseEventContext) -> EventStatus {
        match event {
            MouseEvent::Down { button, position } if button == MouseButton::LEFT && self.is_inside_knob(cx.bounds(), position) => {
                cx.capture_mouse();
                cx.request_render();
                self.last_mouse_pos = Some(position);
                EventStatus::Handled
            },
            MouseEvent::Up { button, .. } if button == MouseButton::LEFT && cx.has_mouse_capture() => {
                cx.release_capture();
                EventStatus::Handled
            },
            MouseEvent::Moved { position, modifiers } => {
                if let Some(last_position) = self.last_mouse_pos {
                    let delta_y = position.y - last_position.y;
                    let delta_value = if modifiers.contains(Modifiers::SHIFT) {
                        delta_y * 0.001
                    } else {
                        delta_y * 0.01
                    };
                    self.normalized_value = (self.normalized_value - delta_value).clamp(0.0, 1.0);
                    cx.request_render();

                    self.last_mouse_pos = Some(position);
                }

                EventStatus::Handled
            },
            _ => EventStatus::Ignored
        }
    }

    fn status_updated(&mut self, event: StatusChange, _cx: &mut crate::app::EventContext) {
        match event {
            StatusChange::MouseCaptureLost => {
                self.last_mouse_pos = None;
            },
            _ => {}
        }
    }

    fn render(&mut self, cx: &mut RenderContext) {
        let bounds = cx.content_bounds();
        let shape = self.shape(bounds);

        let angle = self.current_angle();
        let dot_pos = shape.center + Point::new(angle.cos(), angle.sin()).scale(0.7 * shape.radius);
        cx.fill(shape, Color::GREEN);
        cx.fill(Circle::new(dot_pos, 0.15 * shape.radius), Color::BLACK);
    }
}