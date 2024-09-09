use objc2::rc::Retained;
use objc2_foundation::{ns_string, NSArray, NSMutableArray, NSString};

use crate::param::{ParameterMap, Params};
use crate::platform::audio_toolbox::{AUParameterNode, AudioUnitParameterOptions, AudioUnitParameterUnit};
use crate::platform::mac::audio_toolbox::AUParameterTree;

pub fn create_parameter_tree<P: Params>(parameters: &ParameterMap<P>) -> Retained<AUParameterTree> {
	let mut au_params = NSMutableArray::<AUParameterNode>::new();
	
	for p in parameters.iter() {
		let au_param = AUParameterTree::createParameter(
			&NSString::from_str(p.name()), 
			&NSString::from_str(p.name()), 
			p.id().into(), 
			Into::<f64>::into(p.info().min_value()) as _, 
			Into::<f64>::into(p.info().max_value()) as _, 
			AudioUnitParameterUnit::CustomUnit, 
			ns_string!("Unit"), 
			AudioUnitParameterOptions::IsReadable | AudioUnitParameterOptions::IsWritable, 
			&NSArray::new(), 
			&NSArray::new());
		au_params.push(Retained::into_super(au_param));
	}

	AUParameterTree::createTreeWithChildren(&au_params)
}

