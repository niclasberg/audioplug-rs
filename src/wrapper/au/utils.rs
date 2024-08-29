use objc2::rc::Retained;

use crate::param::Params;
use crate::platform::mac::audio_toolbox::AUParameterTree;

fn create_parameter_tree<P: Params>(p: P) -> Retained<AUParameterTree> {
	todo!()	
}

