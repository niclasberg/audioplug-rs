use vst3::Steinberg::Vst::{
    BusDirection, BusDirections, BusDirections_, BusType, BusTypes, BusTypes_, MediaType,
    MediaTypes, MediaTypes_,
};

use crate::AudioLayout;

pub struct Vst3Busses {
    pub audio_inputs: Vec<Vst3Bus>,
    pub audio_outputs: Vec<Vst3Bus>,
    pub event_inputs: Vec<Vst3Bus>,
    pub event_outputs: Vec<Vst3Bus>,
}

impl Vst3Busses {
    pub fn new(layouts: &[AudioLayout], accepts_midi: bool, produces_midi: bool) -> Self {
        let mut event_inputs = Vec::new();
        if accepts_midi {
            event_inputs.push(Vst3Bus {
                name: "MIDI input",
                channel_count: 1,
                bus_type: BusTypes_::kMain as _,
                media_type: MediaTypes_::kEvent as _,
                default_active: true,
                is_active: true,
                bus_direction: BusDirections_::kInput as _,
            });
        }

        let mut event_outputs = Vec::new();
        if produces_midi {
            event_outputs.push(Vst3Bus {
                name: "MIDI input",
                channel_count: 1,
                bus_type: BusTypes_::kMain as _,
                media_type: MediaTypes_::kEvent as _,
                default_active: true,
                is_active: true,
                bus_direction: BusDirections_::kOutput as _,
            });
        }

        let mut audio_inputs = Vec::new();
        let mut audio_outputs = Vec::new();

        Self {
            audio_inputs,
            audio_outputs,
            event_inputs,
            event_outputs,
        }
    }
}

pub struct Vst3Bus {
    pub name: &'static str,
    pub channel_count: i32,
    pub bus_type: BusType,
    pub media_type: MediaType,
    pub bus_direction: BusDirection,
    pub default_active: bool,
    pub is_active: bool,
}

impl Vst3Bus {}
