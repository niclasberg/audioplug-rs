use crate::platform;

pub struct Clipboard<'a> {
    pub(super) handle: &'a platform::WindowHandle,
}

impl Clipboard<'_> {
    pub fn get_text(&mut self) -> Option<String> {
        self.handle.get_clipboard().ok().flatten()
    }

    pub fn set_text(&mut self, string: &str) {
        self.handle.set_clipboard(string).unwrap();
    }
}
