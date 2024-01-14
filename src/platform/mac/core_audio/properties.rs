use crate::platform::mac::core_foundation::CFString;

use super::{AudioObjectPropertySelector, AudioClassID, AudioObjectID};

trait PropertySelector {
	type Value;
	fn id() -> AudioObjectPropertySelector;
}

macro_rules! property {
	($name:ident, $var: ident, $t:ident) => {
		struct $name;
		impl PropertySelector for $name {
			type Value = $t;
			fn id() -> AudioObjectPropertySelector { unsafe { $var } }
		}
		extern "C" { static $var: AudioObjectPropertySelector; }
	};
}

// AudioObject
property!(AudioObjectPropertyBaseClass, kAudioObjectPropertyBaseClass, AudioClassID);
property!(AudioObjectPropertyClass, kAudioObjectPropertyClass, AudioClassID);
property!(AudioObjectPropertyOwner, kAudioObjectPropertyOwner, AudioObjectID);
property!(AudioObjectPropertyName, kAudioObjectPropertyName, CFString);

// AudioHardware


// AudioDevice
property!(AudioDevicePropertyConfigurationApplication, kAudioDevicePropertyConfigurationApplication, CFString);
property!(AudioDevicePropertyDeviceUID, kAudioDevicePropertyDeviceUID, CFString);

extern "C" {
	// AudioObject
    static kAudioObjectPropertyModelName: AudioObjectPropertySelector;
    static kAudioObjectPropertyManufacturer: AudioObjectPropertySelector;
    static kAudioObjectPropertyElementName: AudioObjectPropertySelector;
    static kAudioObjectPropertyElementCategoryName: AudioObjectPropertySelector;
    static kAudioObjectPropertyElementNumberName: AudioObjectPropertySelector;
    static kAudioObjectPropertyOwnedObjects: AudioObjectPropertySelector;
    static kAudioObjectPropertyIdentify: AudioObjectPropertySelector;
    static kAudioObjectPropertySerialNumber: AudioObjectPropertySelector;
    static kAudioObjectPropertyFirmwareVersion: AudioObjectPropertySelector;

	// AudioHardware
	pub static kAudioHardwarePropertyDevices: AudioObjectPropertySelector;
    pub static kAudioHardwarePropertyDefaultInputDevice: AudioObjectPropertySelector;
    pub static kAudioHardwarePropertyDefaultOutputDevice: AudioObjectPropertySelector;
    pub static kAudioHardwarePropertyDefaultSystemOutputDevice: AudioObjectPropertySelector;
    pub static kAudioHardwarePropertyTranslateUIDToDevice: AudioObjectPropertySelector;
    pub static kAudioHardwarePropertyMixStereoToMono: AudioObjectPropertySelector;
    pub static kAudioHardwarePropertyPlugInList: AudioObjectPropertySelector;
    pub static kAudioHardwarePropertyTranslateBundleIDToPlugIn: AudioObjectPropertySelector;
    pub static kAudioHardwarePropertyTransportManagerList: AudioObjectPropertySelector;
    pub static kAudioHardwarePropertyTranslateBundleIDToTransportManager: AudioObjectPropertySelector;
    pub static kAudioHardwarePropertyBoxList: AudioObjectPropertySelector;
    pub static kAudioHardwarePropertyTranslateUIDToBox: AudioObjectPropertySelector;
    pub static kAudioHardwarePropertyClockDeviceList: AudioObjectPropertySelector;
    pub static kAudioHardwarePropertyTranslateUIDToClockDevice: AudioObjectPropertySelector;
    pub static kAudioHardwarePropertyProcessIsMain: AudioObjectPropertySelector;
    pub static kAudioHardwarePropertyIsInitingOrExiting: AudioObjectPropertySelector;
    pub static kAudioHardwarePropertyUserIDChanged: AudioObjectPropertySelector;
    pub static kAudioHardwarePropertyProcessInputMute: AudioObjectPropertySelector;
    pub static kAudioHardwarePropertyProcessIsAudible: AudioObjectPropertySelector;
    pub static kAudioHardwarePropertySleepingIsAllowed: AudioObjectPropertySelector;
    pub static kAudioHardwarePropertyUnloadingIsAllowed: AudioObjectPropertySelector;
    pub static kAudioHardwarePropertyHogModeIsAllowed: AudioObjectPropertySelector;
    pub static kAudioHardwarePropertyUserSessionIsActiveOrHeadless: AudioObjectPropertySelector;
    pub static kAudioHardwarePropertyServiceRestarted: AudioObjectPropertySelector;
    pub static kAudioHardwarePropertyPowerHint: AudioObjectPropertySelector;
}