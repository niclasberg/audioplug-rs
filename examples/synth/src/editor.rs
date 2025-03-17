use audioplug::{app::View, core::Color, style::Length, views::{Column, Container, Label, ParameterKnob, ParameterSlider, Row, ViewExt}, Editor};

use crate::params::{FilterParams, OscillatorParams, SynthParams};

pub struct SynthEditor {
}

impl Editor for SynthEditor {
    type Parameters = SynthParams;

    fn new() -> Self {
        Self {}
    }

    fn view(&self, parameters: &Self::Parameters) -> impl View {
        Row::new((
            Column::new(parameters.oscillators.iter().map(|p| oscillator_view(p.children())).collect::<Vec<_>>())
                .spacing(Length::Px(5.0))
                .style(|s| s.width(Length::Percent(30.0))),
            Column::new((
                filter_view(&parameters.filter),
            )),
        ))
        .style(|s| s.width(Length::Vw(100.0)).height(Length::Vh(100.0)).background(Color::ASPARAGUS))
    }
}

fn oscillator_view(params: &OscillatorParams) -> impl View {
    Row::new((
        Column::new((
            ParameterKnob::new(&params.octave),
            Label::new("Oct."), 
        )),
        Column::new((
            ParameterKnob::new(&params.semitones)
                .style(|s| s.height(Length::Vh(10.0))),
            Label::new("Semi"), 
        ))
    ))
}

fn filter_view(params: &FilterParams) -> impl View {
    Column::new((
        Row::new((
            Label::new("Cutoff"),
            ParameterSlider::new(&params.cutoff)
        )),
        Row::new((
            Label::new("Resonance"),
            ParameterSlider::new(&params.resonance)
        )),
    ))
}