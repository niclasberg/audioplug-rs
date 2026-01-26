use wayland_client::{Connection, EventQueue, globals::registry_queue_init};

pub struct WaylandApplication {
    connection: Connection,
    event_queue: EventQueue<Self>,
}

impl WaylandApplication {
    pub fn new(connection: Connection) -> Self {
        let (list, event_queue) = registry_queue_init(&connection).unwrap();
        
        Self {connection, event_queue}
    }
}