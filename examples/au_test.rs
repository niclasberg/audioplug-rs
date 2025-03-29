use audioplug::{AudioLayout, GenericEditor, Plugin};
use std::sync::atomic::AtomicBool;

static IS_DONE: AtomicBool = AtomicBool::new(false);

struct TestPlugin {}

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

    fn prepare(&mut self, _sample_rate: f64, _max_samples_per_frame: usize) {}

    fn process(&mut self, _context: audioplug::ProcessContext, _parameters: &()) {}
}

#[cfg(target_os = "macos")]
fn main() {
    use audioplug::wrapper::au::MyAudioUnit;
    use block2::StackBlock;
    use objc2::msg_send;
    use objc2_audio_toolbox::{
        AUAudioUnit, AudioComponentDescription, AudioComponentInstantiationOptions,
    };
    use objc2_avf_audio::AVAudioUnit;
    use objc2_foundation::NSError;

    const fn four_cc(str: &[u8; 4]) -> u32 {
        ((str[0] as u32) << 24 & 0xff000000)
            | ((str[1] as u32) << 16 & 0x00ff0000)
            | ((str[2] as u32) << 8 & 0x0000ff00)
            | ((str[3] as u32) & 0x000000ff)
    }

    let desc: AudioComponentDescription = AudioComponentDescription {
        componentType: four_cc(b"aufx"),
        componentSubType: four_cc(b"demo"),
        componentManufacturer: four_cc(b"Nibe"),
        componentFlags: 0,
        componentFlagsMask: 0,
    };

    // let view_controller: ViewController<TestPlugin> = ViewController::new();
    // view_controller.create_audio_unit(desc, error)

    let audio_unit = MyAudioUnit::new_with_component_descriptor_error(
        TestPlugin::new(),
        desc,
        std::ptr::null_mut(),
    )
    .unwrap();
    let input_busses = unsafe { audio_unit.inputBusses() };
    let output_busses = unsafe { audio_unit.outputBusses() };

    let aa = unsafe { input_busses.count() };
    let bb = unsafe { audio_unit.allocateRenderResourcesAndReturnError() }.unwrap();

    //AUAudioUnit::registerSubclass(MyAudioUnit::<TestPlugin>::class(), desc, ns_string!("RUST: TEST"), 0);

    /*let view_controller_block = StackBlock::new(|view_controller| {
        IS_DONE.store(true, std::sync::atomic::Ordering::Release);
    });*/

    let block = StackBlock::new(move |unit, error: *mut NSError| {
        if let Some(error) = unsafe { error.as_mut() } {
            let aa = error.localizedDescription().to_string();
            println!("{}", aa);
            IS_DONE.store(true, std::sync::atomic::Ordering::Release);
        } else {
            let audio_unit: Option<&AUAudioUnit> = unsafe { msg_send![unit, AUAudioUnit] };
            let provides_user_interface = unsafe { audio_unit.unwrap().providesUserInterface() };

            /*unsafe {
                audio_unit.unwrap().requestViewControllerWithCompletionHandler(&view_controller_block)
            };*/
        }
    });
    unsafe {
        AVAudioUnit::instantiateWithComponentDescription_options_completionHandler(
            desc,
            AudioComponentInstantiationOptions::LoadOutOfProcess,
            &block,
        )
    };

    while !IS_DONE.load(std::sync::atomic::Ordering::Acquire) {}

    println!("asdfas");
}

#[cfg(not(target_os = "macos"))]
fn main() {}
