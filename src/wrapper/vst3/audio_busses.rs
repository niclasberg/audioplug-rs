use vst3_sys::vst::{BusDirection, BusDirections, BusType, BusTypes, MediaType, MediaTypes};

use crate::AudioLayout;

const BUS_TYPE_MAIN: BusType = BusTypes::kMain as _;
const BUS_TYPE_AUX: BusType = BusTypes::kAux as _;
const MEDIA_TYPE_AUDIO: MediaType = MediaTypes::kAudio as _;
const MEDIA_TYPE_EVENT: MediaType = MediaTypes::kEvent as _;
const BUS_DIRECTION_INPUT: BusDirection = BusDirections::kInput as _;
const BUS_DIRECTION_OUTPUT: BusDirection = BusDirections::kOutput as _;

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
                bus_type: BUS_TYPE_MAIN,
                media_type: MEDIA_TYPE_EVENT,
                default_active: true,
                is_active: true,
                bus_direction: BUS_DIRECTION_INPUT,
            });
        }

        let mut event_outputs = Vec::new();
        if produces_midi {
            event_outputs.push(Vst3Bus {
                name: "MIDI input",
                channel_count: 1,
                bus_type: BUS_TYPE_MAIN,
                media_type: MEDIA_TYPE_EVENT,
                default_active: true,
                is_active: true,
                bus_direction: BUS_DIRECTION_OUTPUT,
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
