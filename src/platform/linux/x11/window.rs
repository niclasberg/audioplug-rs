use std::{
    cell::{Cell, RefCell},
    sync::Arc,
};

use x11rb::{
    COPY_DEPTH_FROM_PARENT,
    connection::Connection,
    protocol::xproto::{
        Button, ButtonPressEvent, ButtonReleaseEvent, ConfigureNotifyEvent, ConnectionExt,
        CreateWindowAux, EventMask, ExposeEvent, KeyButMask, MotionNotifyEvent, WindowClass,
        destroy_window,
    },
    reexports::x11rb_protocol::protocol::xproto,
};

use crate::{
    MouseButton, MouseEvent,
    core::{Modifiers, PhysicalCoord, PhysicalSize, Point, Rect, ScaleFactor, Zero},
    platform::{
        Error, WindowEvent, WindowHandler,
        linux::x11::{X11Handle, runloop::X11Runloop},
    },
};

pub struct WindowInner {
    pub(super) id: u32,
    handler: RefCell<Box<dyn WindowHandler>>,
    last_click_pos_and_time: Cell<(Point, u32)>,
    pub physical_size: Cell<PhysicalSize>,
    pub needs_redraw: Cell<bool>,
}

impl WindowInner {
    pub fn new(id: u32, handler: Box<dyn WindowHandler>) -> Self {
        Self {
            id,
            handler: RefCell::new(handler),
            last_click_pos_and_time: Cell::new((Point::ZERO, 0)),
            physical_size: Cell::new(PhysicalSize::ZERO),
            needs_redraw: Cell::new(true),
        }
    }

    pub fn handle_configure(&self, event: ConfigureNotifyEvent) {
        let physical_size = PhysicalSize::new(
            PhysicalCoord(event.width as _),
            PhysicalCoord(event.height as _),
        );
        let old_physical_size = self.physical_size.get();
        if physical_size != old_physical_size {
            self.physical_size.set(physical_size);
            self.handler.borrow_mut().event(WindowEvent::Resize {
                logical_size: physical_size.into_logical(ScaleFactor(1.0)),
                physical_size,
            });
        }
    }

    pub fn handle_expose(&self, event: ExposeEvent) {
        if event.count == 0 {
            self.needs_redraw.set(true);
        }
    }

    pub fn handle_button_press(&self, event: ButtonPressEvent) {
        if let Some(button) = get_mouse_button(event.detail) {
            let position = Point::new(event.event_x as _, event.event_y as _);
            self.handler
                .borrow_mut()
                .event(WindowEvent::Mouse(MouseEvent::Down {
                    button,
                    position,
                    modifiers: get_modifiers(event.state),
                    is_double_click: false,
                }));
            self.last_click_pos_and_time.set((position, event.time));
        }
    }

    pub fn handle_button_release(&self, event: ButtonReleaseEvent) {
        if let Some(button) = get_mouse_button(event.detail) {
            let position = Point::new(event.event_x as _, event.event_y as _);
            self.handler
                .borrow_mut()
                .event(WindowEvent::Mouse(MouseEvent::Up {
                    button,
                    position,
                    modifiers: get_modifiers(event.state),
                }));
        }
    }

    pub fn handle_motion(&self, event: MotionNotifyEvent) {
        let position = Point::new(event.event_x as _, event.event_y as _);
        self.handler
            .borrow_mut()
            .event(WindowEvent::Mouse(MouseEvent::Moved {
                position,
                modifiers: get_modifiers(event.state),
            }));
    }

    pub fn repaint_if_requested(&self) {
        if self.needs_redraw.get() {
            self.handler.borrow_mut().paint(Rect::EMPTY);
            self.needs_redraw.set(false);
        }
    }
}

fn get_mouse_button(button: Button) -> Option<MouseButton> {
    match button {
        1 => Some(MouseButton::LEFT),
        2 => Some(MouseButton::MIDDLE),
        3 => Some(MouseButton::RIGHT),
        _ => None,
    }
}

fn get_modifiers(mask: KeyButMask) -> Modifiers {
    let mut modifiers = Modifiers::empty();
    if mask.contains(KeyButMask::CONTROL) {
        modifiers &= Modifiers::CONTROL;
    }
    if mask.contains(KeyButMask::SHIFT) {
        modifiers &= Modifiers::SHIFT;
    }
    if mask.contains(KeyButMask::MOD1) {
        modifiers &= Modifiers::ALT;
    }
    modifiers
}

pub struct X11Window {
    id: u32,
    runloop: Arc<X11Runloop>,
}

impl Drop for X11Window {
    fn drop(&mut self) {
        self.runloop.unregister_window(self.id);
        destroy_window(&self.runloop.connection, self.id).ok();
    }
}

impl X11Window {
    pub fn open(runloop: Arc<X11Runloop>, handler: Box<dyn WindowHandler>) -> Result<Self, Error> {
        let id = runloop.connection.generate_id().unwrap();
        runloop.register_window(id, WindowInner::new(id, handler));

        let values = CreateWindowAux::default().event_mask(
            EventMask::EXPOSURE
                | EventMask::STRUCTURE_NOTIFY
                | EventMask::BUTTON_PRESS
                | EventMask::BUTTON_RELEASE,
        );
        runloop
            .connection
            .create_window(
                COPY_DEPTH_FROM_PARENT,
                id,
                runloop.screen().root,
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
        x11rb::wrapper::ConnectionExt::change_property32(
            &runloop.connection,
            xproto::PropMode::REPLACE,
            id,
            runloop.atoms.WM_PROTOCOLS,
            xproto::AtomEnum::ATOM,
            &[runloop.atoms.WM_DELETE_WINDOW],
        )
        .unwrap();

        runloop.connection.map_window(id).unwrap();
        runloop.connection.flush().unwrap();
        runloop
            .get_window(id)
            .handler
            .borrow_mut()
            .init(crate::platform::Handle::X11(X11Handle {
                runloop: runloop.clone(),
                id,
            }));

        Ok(Self { runloop, id })
    }

    pub fn attach(
        runloop: &mut X11Runloop,
        parent_window_id: u32,
        handler: Box<dyn WindowHandler>,
    ) -> Result<Self, Error> {
        todo!()
    }
}
