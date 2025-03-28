mod label;
//mod stack;
mod background;
mod button;
mod container;
mod checkbox;
mod filled;
mod image;
mod key_down;
mod knob;
mod scroll;
mod scoped;
mod slider;
mod styled;
mod textbox;
mod xy_pad;
mod util;

pub use background::Background;
pub use button::{Button, ButtonWidget};
pub use checkbox::Checkbox;
pub use container::{Container, Row, Column, Grid};
pub use filled::*;
pub use image::Image;
use key_down::OnKeyEvent;
pub use label::Label;
pub use knob::{KnobWidget, Knob, ParameterKnob};
//pub use scroll::*;
pub use scoped::Scoped;
pub use slider::{Slider, ParameterSlider};
pub use styled::*;
pub use textbox::TextBox;
pub use xy_pad::XyPad;

use crate::{app::{EventStatus, View, WriteContext}, KeyEvent};

pub trait ViewExt {
	fn style(self, f: impl FnOnce(StyleBuilder) -> StyleBuilder) -> Styled<Self> where Self: Sized;
    fn on_key_event<F>(self, f: F) -> OnKeyEvent<Self, F> where 
        Self: Sized, 
        F: Fn(&mut dyn WriteContext, KeyEvent) -> EventStatus + 'static;
}

impl<V: View + Sized> ViewExt for V {
    fn style(self, f: impl FnOnce(StyleBuilder) -> StyleBuilder) -> Styled<Self> {
		let style_builder = f(StyleBuilder::default());
		Styled { view: self, style_builder }
	}
    
    fn on_key_event<F>(self, f: F) -> OnKeyEvent<Self, F> where 
        Self: Sized, 
        F: Fn(&mut dyn WriteContext, KeyEvent) -> EventStatus + 'static 
    {
        OnKeyEvent {
            view: self,
            on_key_down: f,
        }
    }
}