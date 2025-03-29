use windows::Win32::{Foundation::LPARAM, UI::Input::KeyboardAndMouse::*};

use crate::core::{Key, Modifiers};

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

#[derive(Debug)]
pub struct KeyFlags {
    pub repeat_count: u16,
    pub scan_code: u8,
}

impl KeyFlags {
    pub fn from_lparam(lparam: LPARAM) -> Self {
        Self {
            repeat_count: (lparam.0 & 0xFFFF) as u16,
            scan_code: ((lparam.0 >> 16) & 0xFF) as u8,
        }
    }
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
        VK_J => Key::J,
        VK_K => Key::K,
        VK_L => Key::L,
        VK_M => Key::M,
        VK_N => Key::N,
        VK_O => Key::O,
        VK_P => Key::P,
        VK_Q => Key::Q,
        VK_R => Key::R,
        VK_S => Key::S,
        VK_T => Key::T,
        VK_U => Key::U,
        VK_V => Key::V,
        VK_W => Key::W,
        VK_X => Key::X,
        VK_Y => Key::Y,
        VK_Z => Key::Z,

        VK_BACK => Key::BackSpace,
        VK_DELETE => Key::Delete,
        VK_INSERT => Key::Insert,
        VK_HOME => Key::Home,
        VK_END => Key::End,
        VK_PRIOR => Key::PageUp,
        VK_NEXT => Key::PageDown,
        VK_TAB => Key::Tab,
        VK_ESCAPE => Key::Escape,
        VK_RETURN => Key::Enter,

        VK_LEFT => Key::Left,
        VK_RIGHT => Key::Right,
        VK_UP => Key::Up,
        VK_DOWN => Key::Down,

        _ => Key::Unknown,
    }
}
