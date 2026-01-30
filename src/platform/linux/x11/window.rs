use std::rc::Rc;

use x11rb::{
    COPY_DEPTH_FROM_PARENT,
    connection::Connection,
    protocol::xproto::{ConnectionExt, CreateWindowAux, EventMask, WindowClass},
};

use crate::platform::{Error, WindowHandler, linux::x11::X11Application};

pub struct WindowInner {
    id: u32,
    handler: Box<dyn WindowHandler>,
}

pub struct X11Window {
    inner: Rc<WindowInner>,
}

impl X11Window {
    pub fn open(app: &mut X11Application, handler: Box<dyn WindowHandler>) -> Result<Self, Error> {
        let id = app.connection.generate_id().unwrap();
        let values = CreateWindowAux::default().event_mask(EventMask::EXPOSURE);
        app.connection
            .create_window(
                COPY_DEPTH_FROM_PARENT,
                id,
                app.screen().root,
                0,
                0,
                640,
                480,
                0,
                WindowClass::INPUT_OUTPUT,
                0,
                &values,
            )
            .unwrap();

        let inner = Rc::new(WindowInner { handler, id });
        app.register_window(id, inner.clone());

        app.connection.map_window(id).unwrap();

        Ok(Self { inner })
    }
}
