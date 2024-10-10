use audioplug::{AudioLayout, GenericEditor, Plugin};
use audioplug::wrapper::standalone::{standalone_main, AudioProcessor};

struct TestPlugin {

}

impl Plugin for TestPlugin {
    const NAME: &'static str = "test";
    const VENDOR: &'static str = "test";
    const URL: &'static str = "www.test.com";
    const EMAIL: &'static str = "test@test.com";
    const AUDIO_LAYOUT: AudioLayout = AudioLayout::EMPTY;
    type Editor = GenericEditor<()>;
    type Parameters = ();

    fn new() -> Self {
        Self {}
    }

    fn prepare(&mut self, sample_rate: f64, _max_samples_per_frame: usize) {
        
    }

    fn process(&mut self, context: audioplug::ProcessContext, _parameters: &()) {
        todo!()
    }
    
}

fn main() {
    standalone_main::<TestPlugin>()
}