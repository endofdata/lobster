use super::{audio_bus_buffers::AudioBusBuffers, IParameterChanges, IEventList, process_context::ProcessContext};

pub struct ProcessData
{
	process_mode: i32,
	symbolic_sample_size: i32,
	num_samples: i32,
	num_inputs: i32,
	num_outputs: i32,
	inputs: *const AudioBusBuffers,
	outputs: *mut AudioBusBuffers,

	input_parameter_changes: *const IParameterChanges,
	output_parameter_changes:	*const IParameterChanges,
	input_events: *const IEventList,
	output_events: *const IEventList,
	process_context: *const ProcessContext
}
