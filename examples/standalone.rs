use audioplug::{Plugin, GenericEditor};
use audioplug::wrapper::standalone::AudioProcessor;

struct TestPlugin {

}

impl Plugin for TestPlugin {
    const NAME: &'static str = "test";
    const VENDOR: &'static str = "test";
    const URL: &'static str = "www.test.com";
    const EMAIL: &'static str = "test@test.com";
    const AUDIO_LAYOUT: &'static [audioplug::AudioLayout] = &[];
    type Editor = GenericEditor<()>;
    type Parameters = ();

    fn new() -> Self {
        Self {}
    }

    fn reset(&mut self, sample_rate: f64) {
        
    }

    fn process(&mut self, context: audioplug::ProcessContext, _parameters: &()) {
        todo!()
    }
    
}

fn main() {
    let ss = AudioProcessor::<TestPlugin>::new();
    ss.run()
}