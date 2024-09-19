use async_task::Runnable;

use super::Error;

pub struct Executor;

impl Executor {
	pub fn new() -> Result<Self, Error> {
		Ok(Self)
	}

	pub fn dispatch_on_main(&self, _: impl FnOnce() + 'static) {
        
    }

    pub fn dispatch_runnable_on_main<T: 'static>(&self, _: Runnable) {
       
    }
}