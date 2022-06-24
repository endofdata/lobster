use crate::asio_core::*;
use crate::asio_core::sample_buffer::{ SampleBufferFactory, SampleInput, SampleOutput, EmptyBuffer, SamplePanner};

pub enum HardwarePin {
	Input(Box<dyn SampleInput>),
	Output(Box<dyn SampleOutput>)
}

pub struct Channel {
	pub name: String,
	pub pin: HardwarePin
}

pub struct ASIODevice {
	iasio: Option<IASIO>,
	#[allow(dead_code)]
	callbacks: Box<Callbacks>,
	empty_buffer: Box<EmptyBuffer>,
	pub driver_name: String,
	pub input_channels: Box<[Channel]>,
	pub output_channels: Box<[Channel]>
}

// TODO: Really not sure if IASIO is sync safe?
unsafe impl std::marker::Sync for ASIODevice {
}

impl ASIODevice {
	pub fn set_active_device<'a>(clsid : com::CLSID) -> &'a mut ASIODevice{
		unsafe {
			let mut dev = ASIODevice::new();
			dev.open(clsid);
			THE_DEVICE = Some(dev);
			THE_DEVICE.as_mut().unwrap()
		}
	}

	fn new() -> ASIODevice {
		ASIODevice {
			iasio: None,
			callbacks: Box::new(Callbacks {
				buffer_switch: cb_buffer_switch,
				sample_rate_did_change: cb_sample_rate_did_change,
				asio_message: cb_asio_message,
				buffer_switch_time_info: cb_buffer_switch_time_info
			}),
			driver_name: String::from("Null Device"),
			input_channels: Vec::<Channel>::new().into_boxed_slice(),
			output_channels: Vec::<Channel>::new().into_boxed_slice(),
			empty_buffer: Box::new(EmptyBuffer::new(0, true))
		}
	}

	pub fn open(&mut self, clsid: com::CLSID) {
		let iasio = match create_device(&clsid) {
			Ok(value) => value,
			Err(hr) => panic!("Failed to create ASIO device: 0x{:x}", hr),
		};

		self.iasio = Some(iasio);

		let iasio_ref = self.iasio.as_ref().unwrap();
		self.driver_name = ASIODevice::get_driver_name(iasio_ref);	

		let pref_buffer_size = ASIODevice::get_buffer_size(iasio_ref);
		let (max_input_channels, max_output_channels) = ASIODevice::get_channel_count(iasio_ref);
		let num_input_channels = core::cmp::min(max_input_channels, 2);
		let num_output_channels = core::cmp::min(max_output_channels, 2);

		let buffer_infos = ASIODevice::create_buffers(iasio_ref, num_input_channels, num_output_channels, pref_buffer_size, &self.callbacks);

		// TODO: Is it OK we regard only buffers[0]? I think, buffer[1] was same pointer...
		let mut input_channels = Vec::<Channel>::new();
		for index in 0..num_input_channels {
			let buffer_info = &buffer_infos[index as usize];
			input_channels.push(ASIODevice::get_channel(iasio_ref, ASIOBool::True, index, buffer_info, pref_buffer_size));
		}
		self.input_channels = input_channels.into_boxed_slice();

		let mut output_channels = Vec::<Channel>::new();
		for index in 0..num_output_channels {
			let buffer_info = &buffer_infos[(num_input_channels + index) as usize];
			output_channels.push(ASIODevice::get_channel(iasio_ref, ASIOBool::False, index, buffer_info, pref_buffer_size));
		}
		self.output_channels = output_channels.into_boxed_slice();

		self.empty_buffer = Box::new(EmptyBuffer::new(pref_buffer_size as usize, true));
	}

	pub fn set_sample_rate(&mut self, sample_rate: f64) -> bool {
		let iasio_ref = self.iasio.as_ref().unwrap();

		unsafe {
			if iasio_ref.can_sample_rate(sample_rate) == ASIOError::Ok {
				if iasio_ref.set_sample_rate(sample_rate) != ASIOError::Ok{
					panic!("Cannot set desired sample rate '{}'", sample_rate)
				}				
				return self.get_sample_rate() == sample_rate;
			}
			else {
				return false;
			}
		}
	}

	pub fn get_sample_rate(&self) -> f64 {
		let iasio_ref = self.iasio.as_ref().unwrap();

		let mut effective_sample_rate = 0f64;
		unsafe{
			iasio_ref.get_sample_rate(&mut effective_sample_rate);
		}
		effective_sample_rate
	}

	pub fn start(&mut self) {
		let iasio_ref = self.iasio.as_ref().unwrap();
		unsafe {
			iasio_ref.start();
		}
	}

	pub fn stop(&mut self) {
		let iasio_ref = self.iasio.as_ref().unwrap();
		unsafe {
			iasio_ref.stop();
		}
	}

	// pub fn get_features() {
	// 	let selectors = [
	// 		FutureSelector::CanInputMonitor,
	// 		FutureSelector::CanTimeInfo,
	// 		FutureSelector::CanTimeCode,
	// 		FutureSelector::CanTransport,
	// 		FutureSelector::CanInputGain,
	// 		FutureSelector::CanInputMeter,
	// 		FutureSelector::CanOutputGain,
	// 		FutureSelector::CanOutputMeter
	// 	];

	// 	for sel in selectors.iter() {
	// 		result = iasio.future(FutureSelector::EnableTimeCodeRead, core::ptr::null_mut::<()>());
	// 		match result {
	// 			ASIOError::Ok => println!("'{:?}': yes", sel),
	// 			ASIOError::NotPresent => println!("{:?}: no", sel),
	// 			_ => println!("Unexpected answer on querying '{:?}': {:?}", sel, result)
	// 		};
	// 	}
	// }

	// pub fn get_clock_sources()
	// {
	// 	const MAX_CLOCK_SOURCES : usize = 4;
	// 	let mut clock_source_count = MAX_CLOCK_SOURCES as i32;
	// 	let mut clock_sources = [ClockSource::new(); MAX_CLOCK_SOURCES];
		
	// 	if iasio.get_clock_sources(clock_sources.as_mut_ptr(), &mut clock_source_count) != ASIOError::Ok {
	// 		panic!("Failed to get clock source information")
	// 	}
	
	// 	clock_sources.iter().map(|cs| cs.name)
	// }

	fn get_error_message(iasio: &IASIO) -> String {
		let mut buffer = vec![0u8; 256];
		let ptr = buffer.as_mut_ptr();
	
		unsafe {
			iasio.get_error_message(ptr);
		}
		let trimmed : Vec<u8> = buffer.iter().take_while(|c| **c != 0u8).cloned().collect();
	
		String::from_utf8(trimmed)
			.expect("Error message is valid UTF-8")
	}

	fn get_driver_name(iasio: &IASIO) -> String {
		
		let mut driver_info = DriverInfo {
			asio_version: 2,
			driver_version: 0,
			name: [0; 32],
			error_message: [0; 124],
			sys_ref: core::ptr::null::<()>(),
		};

		let driver_info_ptr: *mut DriverInfo = &mut driver_info;

		unsafe {
			match iasio.init(driver_info_ptr as *mut ()) {
				ASIOBool::False => panic!("Driver initialization failed: {}", ASIODevice::get_error_message(&iasio)),
				ASIOBool::True => {
					let mut buffer = vec![0u8; 128];
					let ptr = buffer.as_mut_ptr();
					iasio.get_driver_name(ptr);

					let trimmed : Vec<u8> = buffer.iter().take_while(|c| **c != 0u8).cloned().collect();
					return String::from_utf8(trimmed)
						.expect("Driver name is valid UTF-8");
				}
			}
		}
	}

	fn get_buffer_size(iasio: &IASIO) -> i32 {
		let mut min_buffer_size = 0i32;
		let mut max_buffer_size = 0i32;
		let mut pref_buffer_size = 0i32;
		let mut granularity = 0i32;
		unsafe {
			match iasio.get_buffer_size(&mut min_buffer_size, &mut max_buffer_size, &mut pref_buffer_size, &mut granularity) {
				ASIOError::Ok => pref_buffer_size,
				_ => panic!("Failed to get buffer size information")
			}
		}
	}

	fn get_channel_count(iasio: &IASIO) -> (i32, i32) {
		let mut max_input_channels: i32 = 0;
		let mut max_output_channels: i32 = 0;
	
		unsafe {
			match iasio.get_channels(&mut max_input_channels, &mut max_output_channels) {
				ASIOError::Ok => (max_input_channels, max_output_channels),
				_ => panic!("Failed to get channel count information")
			}
		}
	}

	fn get_channel(iasio: &IASIO, is_input: ASIOBool, id: i32, buffer_info : &BufferInfo, buffer_size: i32) -> Channel {
		let mut channel_info = ChannelInfo::new_for(is_input, id);

		unsafe {
			if iasio.get_channel_info(&mut channel_info) != ASIOError::Ok {
				panic!("Failed to get channel description (is_input: {:?}, id: {})", is_input, id);
			}
		}

		let name = String::from_utf8(channel_info.name.to_vec()).expect("Channel name is utf-8");

		let buffer_a: *mut () = buffer_info.buffers[0];
		let buffer_b: *mut () = buffer_info.buffers[1];

		return match channel_info.sample_type {
			ASIOSampleType::Int32LSB => {
				match is_input {
					ASIOBool::True => Channel { 
						name: name, 
						pin: HardwarePin::Input(SampleBufferFactory::create_input_i32(buffer_a, buffer_b, buffer_size as usize)) 
					},
					ASIOBool::False => Channel {
						name: name,
						pin: HardwarePin::Output(SampleBufferFactory::create_output_i32(buffer_a, buffer_b, buffer_size as usize)) 
					}
				}
			}
			_ => panic!("Unsupported sample type {:?}", channel_info.sample_type)
		};
	}

	fn create_buffers(iasio: &IASIO, num_input_channels : i32, num_output_channels : i32, pref_buffer_size: i32, callbacks: &Callbacks) -> Vec<BufferInfo> {
		let mut buffer_infos = Vec::<BufferInfo>::with_capacity((num_input_channels + num_output_channels) as usize);

		for id in 0..num_input_channels {
			buffer_infos.push(BufferInfo { channel_num: id, is_input: ASIOBool::True, buffers: [core::ptr::null_mut::<()>(); 2] });
		}

		for id in 0..num_output_channels {
			buffer_infos.push(BufferInfo { channel_num: id, is_input: ASIOBool::False, buffers: [core::ptr::null_mut::<()>(); 2] });
		}

		unsafe {
			let result = iasio.create_buffers(buffer_infos.as_mut_ptr(), buffer_infos.len() as i32, pref_buffer_size, callbacks);
			if result != ASIOError::Ok {
				panic!("Failed to create buffers: {:?}", result);
			};
		}
		buffer_infos
	}

	fn buffer_switch(&mut self, params: *const Time, double_buffer_index: i32, _direct_process: ASIOBool) -> *const Time {
		
		// The double_buffer_index indicates, 
		// - which output buffer the host should now start to fill
		// - which input buffer is filled with incoming data by the driver
		let write_second_half = double_buffer_index != 0;
		let read_second_half = double_buffer_index == 0;

		let input_count = self.input_channels.len();

		let input_samples = match input_count {
			1 => {
				let input = self.get_input(0);
				input.select_buffer(read_second_half);
				SamplePanner::new_mono(input.read())
			},
			// 2 => {
			// 	let input_a = self.get_input(0);
			// 	input_a.select_buffer(read_second_half);
			// 	let samples_a = input_a.read();

			// 	let input_b = self.get_input(1);
			// 	input_b.select_buffer(read_second_half);
			// 	let samples_b = input_b.read();

			// 	SamplePanner::new_stereo(samples_a, samples_b)
			// },
			_ => panic!("Unsupported input channel count")
		};

		let output_count = self.output_channels.len();

		match output_count {
			1 => {
				let output_mono = self.get_output(0);
				output_mono.select_buffer(write_second_half);
				output_mono.write(input_samples.mono());
			},
			2 => {
				let output_left = self.get_output(0);
				output_left.select_buffer(write_second_half);
				output_left.write(input_samples.left());

				let output_right = self.get_output(1);
				output_right.select_buffer(write_second_half);
				output_right.write(input_samples.right());
			},
			_ => panic!("Unsupported output channel count")
		};
		params
	}

	fn get_input(&mut self, index: usize) -> &mut dyn SampleInput {
		let input_channels = self.input_channels.as_mut();		
		match &mut input_channels[index].pin {
			HardwarePin::Input(item) => item.as_mut(),
			_ => panic!("Failed to get input pin from input channel.")
		}
	}

	fn get_output(&mut self, index: usize) -> &mut dyn SampleOutput {
		let output_channels = self.output_channels.as_mut();	
		match &mut output_channels[index].pin {
			HardwarePin::Output(item) => item.as_mut(),
			_ => panic!("Failed to get input pin from input channel.")
		}
	}
}

pub static mut THE_DEVICE : Option<ASIODevice> = None;

extern "C" fn cb_buffer_switch(double_buffer_index: i32, direct_process: ASIOBool) {
	cb_buffer_switch_time_info(core::ptr::null::<Time>(), double_buffer_index, direct_process);
}

extern "C" fn cb_buffer_switch_time_info(params: *const Time, double_buffer_index: i32, direct_process: ASIOBool) -> *const Time {
	unsafe {
		match THE_DEVICE.as_mut() {
			Some(dev) => dev.buffer_switch(params, double_buffer_index, direct_process),
			None => params
		}		
	}
}

extern "C" fn cb_sample_rate_did_change(_sample_rate: f64) {
}

extern "C" fn cb_asio_message(selector: MessageSelector, _value: i32, _message: *mut (), _opt: *const f64) -> i32 {
	match selector {
		MessageSelector::SupportsTimeInfo => {
			println!("Supports time info");
			1
		},
		MessageSelector::SupportsTimeCode => {
			println!("Supports time code");
			1
		},
		_ => {
			println!("Unhandled message selector {}", selector as i32);
			0
		}
	}
}