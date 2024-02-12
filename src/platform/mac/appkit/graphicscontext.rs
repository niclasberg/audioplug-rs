use objc2::{extern_class, mutability, ClassType, runtime::NSObject, extern_methods, rc::Id};

use crate::platform::mac::core_graphics::CGContext;

extern_class!(
	#[derive(PartialEq, Eq, Hash)]
	pub struct NSGraphicsContext {
		
	}

	unsafe impl ClassType for NSGraphicsContext {
		type Super = NSObject;
		type Mutability = mutability::InteriorMutable;
	}
);

extern_methods!(
	unsafe impl NSGraphicsContext {
		#[method_id(currentContext)]
        pub fn current() -> Option<Id<NSGraphicsContext>>;

		#[method(CGContext)]
        pub fn cg_context(&self) -> &CGContext;
	}
);
