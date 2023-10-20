use windows::Win32::UI::Input::KeyboardAndMouse::*;

use crate::keyboard::{Key, Modifiers};

pub fn get_modifiers() -> Modifiers {
    let mut modifiers = Modifiers::empty();
    if unsafe { GetKeyState(VK_SHIFT.0.into()) } & 0x80 != 0 {
        modifiers |= Modifiers::SHIFT;
    }
    if unsafe { GetKeyState(VK_CONTROL.0.into()) } & 0x80 != 0 {
        modifiers |= Modifiers::CONTROL;
    }
    modifiers
}

pub fn vk_to_key(vk: VIRTUAL_KEY) -> Key {
    match vk {
        VK_0 => Key::Key0,
        VK_1 => Key::Key1,
        VK_2 => Key::Key2,
        VK_3 => Key::Key3,
        VK_4 => Key::Key4,
        VK_5 => Key::Key5,
        VK_6 => Key::Key6,
        VK_7 => Key::Key7,
        VK_8 => Key::Key8,
        VK_9 => Key::Key9,
        
        VK_A => Key::A,
        VK_B => Key::B,
        VK_C => Key::C,
        VK_D => Key::D,
        VK_E => Key::E,
        VK_F => Key::F,
        VK_G => Key::G,
        VK_H => Key::H,
        VK_I => Key::I,

        _ => Key::Unknown
    }
}