use audioplug::{
    app::{Canvas, PathGeometry, Readable, View},
    core::{Color, ShadowOptions, Size, Vec2},
    param::{AnyParameter, FloatParameter, Parameter},
    style::{AlignItems, Length, UiRect},
    views::{Checkbox, Column, Label, ParameterKnob, ParameterSlider, Row, ViewExt},
    Editor,
};

use crate::params::{AmpEnvelopeParams, FilterParams, OscillatorParams, SynthParams};

const PADDING: UiRect = UiRect::all_px(5.0);
const SPACER: Length = Length::Px(5.0);
const SHADOW: ShadowOptions = ShadowOptions {
    radius: 6.0,
    offset: Vec2::splat(2.0),
    ..ShadowOptions::DEFAULT
};

pub struct SynthEditor {}

impl Editor for SynthEditor {
    type Parameters = SynthParams;

    fn new() -> Self {
        Self {}
    }

    fn view(&self, parameters: &Self::Parameters) -> impl View {
        Column::new((header_view(), main_view(parameters))).style(|s| {
            s.width(Length::Vw(100.0))
                .height(Length::Vh(100.0))
                .background(Color::ASPARAGUS)
        })
    }
}

fn header_view() -> impl View {
    Row::new((Label::new("hello"),)).style(|s| {
        s.width(Length::Percent(100.0))
            .background(Color::BITTER_LEMON)
            .box_shadow(SHADOW)
    })
}

fn main_view(parameters: &SynthParams) -> impl View {
    let oscillator_views: Vec<_> = parameters
        .oscillators
        .iter()
        .map(|p| oscillator_view(p.children()))
        .collect();

    Row::new((
        Column::new(oscillator_views)
            .spacing(SPACER)
            .style(|s| s.width(Length::Percent(30.0))),
        Column::new((
            filter_view(&parameters.filter),
            amp_envelope_view(&parameters.envelope),
        ))
        .spacing(SPACER),
    ))
    .spacing(SPACER)
    .style(|s| s.padding(PADDING))
}

fn oscillator_view(params: &OscillatorParams) -> impl View {
    Row::new((
        Checkbox::new(),
        Column::new((ParameterKnob::new(&params.octave), Label::new("Oct."))),
        Column::new((ParameterKnob::new(&params.semitones), Label::new("Semi"))),
    ))
    .spacing(SPACER)
    .v_align_center()
}

fn filter_view(params: &FilterParams) -> impl View {
    Row::new((
        Column::new((
            ParameterSlider::new(&params.cutoff)
                .vertical()
                .style(|s| s.height(Length::Px(120.0))),
            Label::new("Cutoff"),
        )),
        Column::new((
            ParameterSlider::new(&params.resonance)
                .vertical()
                .style(|s| s.height(Length::Px(120.0))),
            Label::new("Resonance"),
        )),
    ))
    .spacing(SPACER)
    .style(|s| {
        s.padding(PADDING)
            .corner_radius(Size::new(5.0, 5.0))
            .background(Color::BLACK.with_alpha(0.2))
            .box_shadow(SHADOW)
    })
}

fn amp_envelope_view(params: &AmpEnvelopeParams) -> impl View {
    Column::new((
        Label::new("Amp. envelope"),
        envelope_graph(
            &params.attack,
            &params.decay,
            &params.sustain,
            &params.release,
        ),
        Row::new((
            Column::new((
                ParameterKnob::new(&params.attack),
                Label::new("A").style(|s| s.align_self(AlignItems::Center)),
            )),
            Column::new((
                ParameterKnob::new(&params.decay),
                Label::new("D").style(|s| s.align_self(AlignItems::Center)),
            )),
            Column::new((
                ParameterKnob::new(&params.sustain),
                Label::new("S").style(|s| s.align_self(AlignItems::Center)),
            )),
            Column::new((
                ParameterKnob::new(&params.release),
                Label::new("R").style(|s| s.align_self(AlignItems::Center)),
            )),
        ))
        .spacing(SPACER),
    ))
    .spacing(SPACER)
    .style(|s| {
        s.padding(PADDING)
            .corner_radius(Size::new(5.0, 5.0))
            .background(Color::BLACK.with_alpha(0.2))
            .box_shadow(SHADOW)
    })
}

fn envelope_graph(
    attack: &FloatParameter,
    decay: &FloatParameter,
    sustain: &FloatParameter,
    release: &FloatParameter,
) -> impl View {
    let a = attack.as_signal();
    let d = decay.as_signal();
    let s = sustain.as_signal();
    let r = release.as_signal();
    let max_env_time =
        attack.info().max_value().0 + decay.info().max_value().0 + release.info().max_value().0;

    Canvas::new(move |cx, _| {
        let bounds = cx.bounds();
        let s_width = 0.2;
        let a_d_r_width = (1.0 - s_width) / max_env_time;
        let a_end = a.get(cx) * a_d_r_width;
        let d_end = a_end + d.get(cx) * a_d_r_width;
        let s_end = d_end + s_width;
        let r_end = s_end + r.get(cx) * a_d_r_width;

        let geometry = PathGeometry::new(|b| {
            b.move_to(bounds.get_relative_point(0.0, 1.0))
                .add_line_to(bounds.get_relative_point(a_end, 0.0))
                .add_line_to(bounds.get_relative_point(d_end, 1.0 - s.get(cx)))
                .add_line_to(bounds.get_relative_point(s_end, 1.0 - s.get(cx)))
                .add_line_to(bounds.get_relative_point(r_end, 1.0))
                .close()
        });

        cx.fill(&geometry, Color::BLACK);
    })
    .style(|s| {
        s.background(Color::WHITE.with_alpha(0.2))
            .padding(UiRect::all_px(2.0))
            .width(Length::Percent(100.0))
            .height(Length::Px(30.0))
    })
}
