use wayland_client::{Connection, Dispatch, EventQueue, globals::{GlobalList, GlobalListContents, registry_queue_init}, protocol::{wl_display::WlDisplay, wl_registry}};

pub struct WaylandApplication {
    connection: Connection,
    event_queue: EventQueue<Self>,
    globals: GlobalList
}

impl WaylandApplication {
    pub fn new(connection: Connection) -> Self {
        let (globals, event_queue) = registry_queue_init(&connection).unwrap();
        globals.contents().with_list(|list| {
            for entry in list {
                println!("{:?}", entry)
            }
        });
        
        Self {connection, event_queue, globals}
    }

    pub fn display(&self) -> WlDisplay {
        self.connection.display()
    }

    pub fn run(&mut self) {
        
    }
}

impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for WaylandApplication {
    fn event(
        _state: &mut Self,
        _proxy: &wl_registry::WlRegistry,
        _event: <wl_registry::WlRegistry as wayland_client::Proxy>::Event,
        _data: &GlobalListContents,
        _conn: &Connection,
        _qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        
    }
}