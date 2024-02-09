use crate::{Plugin, platform::AudioHost};

pub struct AudioProcessor<P> {
    plugin: P
}

impl<P: Plugin> AudioProcessor<P> {
    pub fn new() -> Self {
        Self { plugin: P::new() }
    }

    pub fn run(mut self) {
        for device in AudioHost::devices().unwrap() {
            println!("{:?}", device.name())
        }

        let output_device = AudioHost::default_output_device().unwrap();
        let sample_rate = output_device.sample_rate().unwrap();

        self.plugin.reset(sample_rate as f64);

    }
}

