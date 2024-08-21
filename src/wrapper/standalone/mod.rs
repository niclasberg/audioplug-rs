use std::{cell::RefCell, rc::Rc};
use rtrb::{Consumer, Producer, RingBuffer};

use crate::{app::HostHandle, param::{NormalizedValue, ParameterId}, platform::AudioHost, App, Editor, Plugin};

const SAMPLES_PER_BLOCK: usize = 128;

pub struct ParameterUpdate {
    id: ParameterId,
    value: NormalizedValue
}

struct AppInner {
    parameter_updates: Producer<ParameterUpdate>,
}

struct StandaloneHostHandler {
    app_inner: Rc<RefCell<AppInner>>,
}

impl HostHandle for StandaloneHostHandler {
    fn begin_edit(&self, id: crate::param::ParameterId) {
        
    }

    fn end_edit(&self, id: crate::param::ParameterId) {
        
    }

    fn perform_edit(&self, id: crate::param::ParameterId, value: crate::param::NormalizedValue) {
        let mut app_inner = RefCell::borrow_mut(&self.app_inner);
        app_inner.parameter_updates.push(ParameterUpdate {id, value }).unwrap();
    }
}

pub struct StandaloneApp<P: Plugin> {
    app: App,
    editor: P::Editor,
    parameters: P::Parameters,
}

impl<P: Plugin> StandaloneApp<P> {
    pub fn new() -> Self {
        let app = App::new();
        let editor = P::Editor::new();
        let parameters = P::Parameters::default();
        //let window = Window::open(&mut app, |)
        Self {
            app,
            editor,
            parameters
        }
    }
}

pub struct AudioProcessor<P> {
    plugin: P,
    parameter_updates: Consumer<ParameterUpdate>,
}

impl<P: Plugin> AudioProcessor<P> {
    pub fn new(parameter_updates: Consumer<ParameterUpdate>) -> Self {
        Self { 
            plugin: P::new(),
            parameter_updates
        }
    }

    pub fn run(mut self) {
        for device in AudioHost::devices().unwrap() {
            println!("{:?}", device.name())
        }

        let output_device = AudioHost::default_output_device().unwrap();
        let sample_rate = output_device.sample_rate().unwrap();

        self.plugin.reset(sample_rate as f64, SAMPLES_PER_BLOCK);
    }
}

pub fn standalone_main<P: Plugin>() {
    let (producer, consumer) = RingBuffer::new(1024);
    let processor = AudioProcessor::<P>::new(consumer);
    processor.run();
}