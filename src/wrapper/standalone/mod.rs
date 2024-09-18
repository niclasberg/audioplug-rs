use std::{cell::RefCell, rc::Rc};
use rtrb::{Consumer, Producer, RingBuffer};

use crate::{app::{AppState, HostHandle, Window}, param::{NormalizedValue, ParameterId, ParameterInfo}, platform::{self, AudioHost}, App, Editor, Plugin};

const SAMPLES_PER_BLOCK: usize = 128;

pub struct ParameterUpdate {
    id: ParameterId,
    value: NormalizedValue
}

struct AppInner {
    parameter_updates: Producer<ParameterUpdate>,
}

struct StandaloneHostHandle {
    app_inner: Rc<RefCell<AppInner>>,
}

impl HostHandle for StandaloneHostHandle {
    fn begin_edit(&self, id: ParameterId) {
        
    }

    fn end_edit(&self, id: ParameterId) {
        
    }

    fn perform_edit(&self, info: &dyn ParameterInfo, value: crate::param::NormalizedValue) {
        let mut app_inner = RefCell::borrow_mut(&self.app_inner);
        app_inner.parameter_updates.push(ParameterUpdate {id: info.id(), value }).unwrap();
    }
}

pub struct StandaloneApp<P: Plugin> {
    app: App,
    editor: P::Editor,
}

impl<P: Plugin> StandaloneApp<P> {
    pub fn new(parameter_updates: Producer<ParameterUpdate>, executor: Rc<platform::Executor>) -> Self {
        let app_inner = Rc::new(RefCell::new(AppInner { parameter_updates }));
        let host_handle = StandaloneHostHandle { app_inner: app_inner.clone() };
        let mut app_state = AppState::new(P::Parameters::default(), executor);
        app_state.set_host_handle(Some(Box::new(host_handle)));

        let state = Rc::new(RefCell::new(app_state));
        let app = App::new_with_app_state(state);
        let editor = P::Editor::new();
        Self {
            app,
            editor
        }
    }

    pub fn run(mut self) {
        let editor = self.editor;
        let _ = Window::open(&mut self.app, move |ctx| {
            let p = P::Parameters::default();
            editor.view(ctx, &p)
        });
        self.app.run()
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

    pub fn start(mut self) {
        for device in AudioHost::devices().unwrap() {
            println!("{:?}", device.name())
        }

        let output_device = AudioHost::default_output_device().unwrap();
        let sample_rate = output_device.sample_rate().unwrap();

        self.plugin.reset(sample_rate as f64, SAMPLES_PER_BLOCK);
    }
}

pub fn standalone_main<P: Plugin>() {
    let executor = Rc::new(platform::Executor::new().unwrap());
    let (producer, consumer) = RingBuffer::new(1024);
    let processor = AudioProcessor::<P>::new(consumer);
    let app = StandaloneApp::<P>::new(producer, executor);
    processor.start();
    app.run();
}