//use crate::asio_core::{ ASIOBool, ASIOError, ASIOSampleType, create_device, IASIO, DriverInfo, ChannelInfo, BufferInfo, Time, MessageSelector, Callbacks };
use crate::asio_core::*;
use crate::asio_core::sample_buffer::{ SampleBufferFactory, SampleInput, SampleOutput};

pub enum HardwarePin {
	Input(Box<dyn SampleInput>),
	Output(Box<dyn SampleOutput>)
}

pub struct Channel {
	pub name: String,
	pub pin: HardwarePin
}

pub struct ASIODevice {
	iasio: IASIO,
	#[allow(dead_code)]
	callbacks: Box<Callbacks>,
	pub driver_name: String,
	pub input_channels: Vec<Channel>,
	pub output_channels: Vec<Channel>
}

impl ASIODevice {
	pub fn new(clsid: com::CLSID) -> ASIODevice {

		let iasio = match create_device(&clsid) {
			Ok(value) => value,
			Err(hr) => panic!("Failed to create ASIO device: 0x{:x}", hr),
		};

		let driver_name = ASIODevice::get_driver_name(&iasio);		
		let pref_buffer_size = ASIODevice::get_buffer_size(&iasio);
		let (max_input_channels, max_output_channels) = ASIODevice::get_channel_count(&iasio);
		let num_input_channels = core::cmp::min(max_input_channels, 2);
		let num_output_channels = core::cmp::min(max_output_channels, 2);
		let callbacks = Box::new(Callbacks {
			buffer_switch: cb_buffer_switch,
			sample_rate_did_change: cb_sample_rate_did_change,
			asio_message: cb_asio_message,
			buffer_switch_time_info: cb_buffer_switch_time_info
		});

		let buffer_infos = ASIODevice::create_buffers(&iasio, num_input_channels, num_output_channels, pref_buffer_size, &callbacks);

		// TODO: Is it OK we regard only buffers[0]? I think, buffer[1] was same pointer...
		let mut input_channels = Vec::<Channel>::new();
		for index in 0..num_input_channels {
			input_channels.push(ASIODevice::get_channel(&iasio, ASIOBool::True, index, buffer_infos[index as usize].buffers[0], pref_buffer_size));
		}

		let mut output_channels = Vec::<Channel>::new();
		for index in 0..num_output_channels {
			output_channels.push(ASIODevice::get_channel(&iasio, ASIOBool::False, index, buffer_infos[(num_input_channels + index) as usize].buffers[0], pref_buffer_size));
		}		

		return ASIODevice {
			iasio : iasio,
			callbacks: callbacks,
			driver_name : driver_name,
			input_channels: input_channels,
			output_channels: output_channels
		}	
	}

	pub fn set_sample_rate(&mut self, sample_rate: f64) -> bool {
		unsafe {
			if self.iasio.can_sample_rate(sample_rate) == ASIOError::Ok {
				if self.iasio.set_sample_rate(sample_rate) != ASIOError::Ok{
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
		let mut effective_sample_rate = 0f64;
		unsafe{
			self.iasio.get_sample_rate(&mut effective_sample_rate);
		}
		effective_sample_rate
	}

	pub fn start(&mut self) {
		unsafe {
			self.iasio.start();
		}
	}

	pub fn stop(&mut self) {
		unsafe {
			self.iasio.stop();
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

	fn get_channel(iasio: &IASIO, is_input: ASIOBool, id: i32, buffer: *mut (), buffer_size: i32) -> Channel {
		let mut channel_info = ChannelInfo::new_for(is_input, id);

		unsafe {
			if iasio.get_channel_info(&mut channel_info) != ASIOError::Ok {
				panic!("Failed to get channel description (is_input: {:?}, id: {})", is_input, id);
			}
		}

		let name = String::from_utf8(channel_info.name.to_vec()).expect("Channel name is utf-8");

		return match channel_info.sample_type {
			ASIOSampleType::Int32LSB => {
				match is_input {
					ASIOBool::True => Channel { 
						name: name, 
						pin: HardwarePin::Input(SampleBufferFactory::create_input_i32(buffer, buffer_size as usize)) 
					},
					ASIOBool::False => Channel {
						name: name,
						pin: HardwarePin::Output(SampleBufferFactory::create_output_i32(buffer, buffer_size as usize)) 
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

		// for &info in &buffer_infos {
		// 	println!("{:?} {} {:x} {:x}", info.is_input, info.channel_num, info.buffers[0] as usize, info.buffers[1] as usize);
		// }

		// let fred = buffer_infos.as_mut_ptr();
		// unsafe {
		// 	let lutz = fred.offset(1);
		// 	let info : &BufferInfo = &*lutz;

		// 	println!("{:?} {} {:x} {:x}", info.is_input, info.channel_num, info.buffers[0] as usize, info.buffers[1] as usize);
		// }
		buffer_infos
	}
}

extern "C" fn cb_buffer_switch(double_buffer_index: i32, direct_process: ASIOBool) {
	cb_buffer_switch_time_info(core::ptr::null::<Time>(), double_buffer_index, direct_process);
}

extern "C" fn cb_buffer_switch_time_info(_params: *const Time, _double_buffer_index: i32, _direct_process: ASIOBool) -> *const Time {
	_params
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