use crate::midi::NoteEvent;

pub struct MidiBuffer {
	events: Box<[NoteEvent]>,
	tail_index: usize
}

impl MidiBuffer {
	pub fn new(capacity: usize) -> Self {
		Self {
			events: Box::new([]),
			tail_index: 0
		}
	}

	pub fn reset(&mut self) {
		self.tail_index = 0;
	}

	pub fn push(&mut self, event: NoteEvent) -> bool {
		if self.tail_index >= self.events.len() {
			return false;
		}
		self.events[self.tail_index] = event;
		self.tail_index += 1;
		true
	}
}