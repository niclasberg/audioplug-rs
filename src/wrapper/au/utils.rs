use objc2::rc::Retained;
use objc2_foundation::{ns_string, NSArray, NSMutableArray, NSString};

use crate::param::{AnyParameter, AnyParameterGroup, ParamVisitor, ParameterMap, Params};
use crate::platform::audio_toolbox::{AUParameterNode, AudioUnitParameterOptions, AudioUnitParameterUnit};
use crate::platform::mac::audio_toolbox::AUParameterTree;

struct CreateParametersVisitor {
	au_params: Retained<NSMutableArray<AUParameterNode>>
}

impl CreateParametersVisitor {
	pub fn new() -> Self {
		Self {
			au_params: NSMutableArray::<AUParameterNode>::new(),
		}	
	}
}

impl ParamVisitor for CreateParametersVisitor {
	fn bool_parameter(&mut self, p: &crate::param::BoolParameter) {
		let au_param = AUParameterTree::createParameter(
			&NSString::from_str(p.info().name()), 
			&NSString::from_str(p.info().name()), 
			p.info().id().into(), 
			Into::<f64>::into(p.info().min_value()) as _, 
			Into::<f64>::into(p.info().max_value()) as _, 
			AudioUnitParameterUnit::Boolean, 
			ns_string!("-"), 
			AudioUnitParameterOptions::IsReadable | AudioUnitParameterOptions::IsWritable, 
			&NSArray::new(), 
			&NSArray::new());
		self.au_params.push(Retained::into_super(au_param));
	}

	fn bypass_parameter(&mut self, p: &crate::param::ByPassParameter) {
		let au_param = AUParameterTree::createParameter(
			&NSString::from_str(p.info().name()), 
			&NSString::from_str(p.info().name()), 
			p.info().id().into(), 
			Into::<f64>::into(p.info().min_value()) as _, 
			Into::<f64>::into(p.info().max_value()) as _, 
			AudioUnitParameterUnit::Boolean, 
			ns_string!("-"), 
			AudioUnitParameterOptions::IsReadable | AudioUnitParameterOptions::IsWritable, 
			&NSArray::new(), 
			&NSArray::new());
		self.au_params.push(Retained::into_super(au_param));
	}

	fn float_parameter(&mut self, p: &crate::param::FloatParameter) {
		let au_param = AUParameterTree::createParameter(
			&NSString::from_str(p.info().name()), 
			&NSString::from_str(p.info().name()), 
			p.info().id().into(), 
			Into::<f64>::into(p.info().min_value()) as _, 
			Into::<f64>::into(p.info().max_value()) as _, 
			AudioUnitParameterUnit::CustomUnit, 
			ns_string!("Unit"), 
			AudioUnitParameterOptions::IsReadable | AudioUnitParameterOptions::IsWritable, 
			&NSArray::new(), 
			&NSArray::new());
		self.au_params.push(Retained::into_super(au_param));
	}

	fn int_parameter(&mut self, p: &crate::param::IntParameter) {
		let au_param = AUParameterTree::createParameter(
			&NSString::from_str(p.info().name()), 
			&NSString::from_str(p.info().name()), 
			p.info().id().into(), 
			Into::<f64>::into(p.info().min_value()) as _, 
			Into::<f64>::into(p.info().max_value()) as _, 
			AudioUnitParameterUnit::CustomUnit, 
			ns_string!("Unit"), 
			AudioUnitParameterOptions::IsReadable | AudioUnitParameterOptions::IsWritable, 
			&NSArray::new(), 
			&NSArray::new());
		self.au_params.push(Retained::into_super(au_param));
	}

	fn string_list_parameter(&mut self, p: &crate::param::StringListParameter) {
		todo!()
	}

	fn group<P: crate::param::ParameterTraversal>(&mut self, group: &crate::param::ParameterGroup<P>) {
		let mut child_visitor = Self::new();
		group.children().visit(&mut child_visitor);
		let au_group = AUParameterTree::createGroupWithIdentifier(
			&NSString::from_str(group.name()), 
			&NSString::from_str(group.name()), 
			&child_visitor.au_params);
		self.au_params.push(Retained::into_super(au_group));
	}
}

pub fn create_parameter_tree<P: Params>(parameters: &ParameterMap<P>) -> Retained<AUParameterTree> {
	let mut visitor = CreateParametersVisitor::new();
	parameters.parameters_ref().visit(&mut visitor);
	AUParameterTree::createTreeWithChildren(&visitor.au_params)
}

