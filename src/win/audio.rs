use windows::{
    core::*,
    Win32::{
        Media::Audio, 
        System::Com,
        Devices::Properties,
        System::Threading::{CreateEventExW, EVENT_MODIFY_STATE, SYNCHRONIZATION_SYNCHRONIZE, CREATE_EVENT_MANUAL_RESET}, UI::Shell::PropertiesSystem::PropVariantGetStringElem
    }
};

use super::com;

struct AudioClientWrapper {
    audio_client: Audio::IAudioClient
}

impl AudioClientWrapper {
    
}

pub struct Device {
    device: Audio::IMMDevice
}

struct Enumerator(Audio::IMMDeviceEnumerator);

impl Device {
    pub fn default_output_device() -> Device {
        todo!()
    }

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

    pub fn new() -> Result<Self> {
        com::com_initialized();
        let device_enumerator: Audio::IMMDeviceEnumerator = unsafe { Com::CoCreateInstance(&Audio::MMDeviceEnumerator, None, Com::CLSCTX_ALL)? };
        //let device_collection: IMMDeviceCollection = unsafe {device_enumerator.EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE)? };
        let endpoint = unsafe { device_enumerator.GetDefaultAudioEndpoint(Audio::eRender, Audio::eConsole)? };
        let audio_client = unsafe { endpoint.Activate::<Audio::IAudioClient>(Com::CLSCTX_INPROC_SERVER, None)? };

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
    }
}