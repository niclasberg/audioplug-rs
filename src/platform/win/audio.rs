use std::{sync::OnceLock, cell::{RefCell, OnceCell}, rc::Rc, thread::JoinHandle};

use windows::{
    core::*,
    Win32::{
        Media::Audio, 
        System::Com::{self, StructuredStorage::PropVariantGetStringElem},
        Devices::Properties,
        System::Threading::{CreateEventExW, EVENT_MODIFY_STATE, SYNCHRONIZATION_SYNCHRONIZE, CREATE_EVENT_MANUAL_RESET}
    }
};

use super::com;

pub struct AudioHost;

impl AudioHost {
    pub fn default_input_device() -> Result<AudioDevice> {
        let device = unsafe { device_enumerator().GetDefaultAudioEndpoint(Audio::eCapture, Audio::eConsole)? };
        Ok(AudioDevice::new(device))
    }

    pub fn default_output_device() -> Result<AudioDevice> {
        let device = unsafe { device_enumerator().GetDefaultAudioEndpoint(Audio::eRender, Audio::eConsole)? };
        Ok(AudioDevice::new(device))
    }

    pub fn devices() -> Result<Vec<AudioDevice>> {
        let endpoints = unsafe { device_enumerator().EnumAudioEndpoints(Audio::eAll, Audio::DEVICE_STATE_ACTIVE)? };
        let count = unsafe { endpoints.GetCount()? };
        (0..count).map(|i| {
            let device = unsafe { endpoints.Item(i)? };
            Ok(AudioDevice::new(device))
        }).collect()
    }
}

pub struct AudioDevice {
    device: Audio::IMMDevice,
    audio_client: OnceCell<Audio::IAudioClient>,
}

impl AudioDevice {
    pub fn id(&self) -> Result<String> {
        let id = unsafe { self.device.GetId()? };
        Ok(unsafe { id.to_string()? })
    }

    pub fn name(&self) -> Result<String> {
        let property_store = unsafe { self.device.OpenPropertyStore(Com::STGM_READ)? };
        let friendly_name = unsafe { property_store.GetValue(&Properties::DEVPKEY_Device_FriendlyName as *const _ as *const _)? };
        let str = unsafe { PropVariantGetStringElem(&friendly_name, 0)? };
        Ok(unsafe { str.to_string()? })
    }

    fn get_audio_client(&self) -> Result<&Audio::IAudioClient> {
        if let Some(audio_client) = self.audio_client.get() {
            Ok(audio_client)
        } else {
            let audio_client = unsafe { self.device.Activate::<Audio::IAudioClient>(Com::CLSCTX_INPROC_SERVER, None)? };
            self.audio_client.set(audio_client).unwrap();
            Ok(self.audio_client.get().unwrap())
        }
    }

    pub fn sample_rate(&self) -> Result<u32> {
        let audio_client = self.get_audio_client()?;
        unsafe {
            let mix_format = audio_client.GetMixFormat()?;
            Ok((*mix_format).nSamplesPerSec)
        }
    }

    pub fn new(device: Audio::IMMDevice) -> Self {
        Self { device, audio_client: OnceCell::new() }
    }

    pub fn create_output_stream(&self) -> Result<Stream> {
        todo!()
    }

    /*pub fn new2() -> Result<Self> {
        //let device_collection: IMMDeviceCollection = unsafe {device_enumerator.EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE)? };
        let endpoint = unsafe { device_enumerator().GetDefaultAudioEndpoint(Audio::eRender, Audio::eConsole)? };  

        let mix_format = unsafe { audio_client.GetMixFormat()? };

        let latency = 40;
        unsafe {
            audio_client.Initialize(
                Audio::AUDCLNT_SHAREMODE_SHARED, 
                Audio::AUDCLNT_STREAMFLAGS_EVENTCALLBACK | Audio::AUDCLNT_STREAMFLAGS_NOPERSIST, 
                latency * 10_000, 
                0, 
                mix_format, 
                None)?
        };
        let sample_rate = unsafe { (*mix_format).nSamplesPerSec };
        let n_channels = unsafe { (*mix_format).nChannels };
        let render_client = unsafe { audio_client.GetService::<Audio::IAudioRenderClient>()? };
        //audio_client.GetService()

        let sample_ready_event = unsafe { CreateEventExW(None, None, CREATE_EVENT_MANUAL_RESET, (EVENT_MODIFY_STATE | SYNCHRONIZATION_SYNCHRONIZE).0)? };

        unsafe { audio_client.SetEventHandle(sample_ready_event)? };

        Ok(Self {
            device: endpoint
        })
    }*/
}

struct Enumerator(Audio::IMMDeviceEnumerator);
unsafe impl Sync for Enumerator {}
unsafe impl Send for Enumerator {}

fn device_enumerator() -> &'static Audio::IMMDeviceEnumerator {
    static INSTANCE: OnceLock<Enumerator> = OnceLock::new();
    let enumerator = INSTANCE.get_or_init(|| {
        com::com_initialized();
        let enumerator = unsafe { Com::CoCreateInstance(&Audio::MMDeviceEnumerator, None, Com::CLSCTX_ALL) }.unwrap();
        Enumerator(enumerator)
    });
    &enumerator.0
}

pub struct Stream {
    thread_handle: Option<JoinHandle<()>>
}

impl Drop for Stream {
    fn drop(&mut self) {
        self.thread_handle.take().unwrap().join().unwrap()
    }
}