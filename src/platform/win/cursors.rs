use windows::core::Result;
use windows::Win32::UI::WindowsAndMessaging::{
    LoadCursorW, HCURSOR, IDC_ARROW, IDC_CROSS, IDC_HAND, IDC_HELP, IDC_IBEAM, IDC_NO,
    IDC_SIZENESW, IDC_SIZENS, IDC_SIZENWSE, IDC_SIZEWE, IDC_WAIT,
};

use crate::core::Cursor;

pub struct Cursors {
    arrow: HCURSOR,
    hand: HCURSOR,
    help: HCURSOR,
    ibeam: HCURSOR,
    not_allowed: HCURSOR,
    wait: HCURSOR,
    cross: HCURSOR,
    up_down: HCURSOR,
    left_right: HCURSOR,
    left_up_right_down: HCURSOR,
    left_down_right_up: HCURSOR,
}

impl Cursors {
    pub fn new() -> Result<Self> {
        let arrow = unsafe { LoadCursorW(None, IDC_ARROW) }?;
        let hand = unsafe { LoadCursorW(None, IDC_HAND) }?;
        let help = unsafe { LoadCursorW(None, IDC_HELP) }?;
        let ibeam = unsafe { LoadCursorW(None, IDC_IBEAM) }?;
        let not_allowed = unsafe { LoadCursorW(None, IDC_NO) }?;
        let wait = unsafe { LoadCursorW(None, IDC_WAIT) }?;
        let cross = unsafe { LoadCursorW(None, IDC_CROSS) }?;
        let up_down = unsafe { LoadCursorW(None, IDC_SIZENS) }?;
        let left_right = unsafe { LoadCursorW(None, IDC_SIZEWE) }?;
        let left_up_right_down = unsafe { LoadCursorW(None, IDC_SIZENWSE) }?;
        let left_down_right_up = unsafe { LoadCursorW(None, IDC_SIZENESW) }?;

        Ok(Self {
            arrow,
            hand,
            help,
            ibeam,
            not_allowed,
            wait,
            cross,
            up_down,
            left_right,
            left_up_right_down,
            left_down_right_up,
        })
    }

    pub fn get_cursor(&self, cursor: Cursor) -> HCURSOR {
        match cursor {
            Cursor::Arrow => self.arrow,
            Cursor::Hand => self.hand,
            Cursor::Help => self.help,
            Cursor::IBeam => self.ibeam,
            Cursor::NotAllowed => self.not_allowed,
            Cursor::Wait => self.wait,
            Cursor::Cross => self.cross,
            Cursor::UpDown => self.up_down,
            Cursor::LeftRight => self.left_right,
            Cursor::LeftUpRightDown => self.left_up_right_down,
            Cursor::LeftDownRightUp => self.left_down_right_up,
        }
    }
}

thread_local! { static CURSORS: Cursors = {
    Cursors::new().unwrap()
}}

pub fn get_cursor(cursor: Cursor) -> HCURSOR {
    CURSORS.with(|cursors| cursors.get_cursor(cursor))
}
