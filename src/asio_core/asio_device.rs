use crate::asio_core::{ IASIO, Callbacks, ASIOBool, ASIOError, ASIOSampleType, DriverInfo, BufferInfo, ChannelInfo, Time, MessageSelector, create_device };
use crate::asio_core::input_channel::InputChannel;
use crate::asio_core::output_channel::OutputChannel;

pub trait ASIODeviceType {
	fn buffer_switch(&mut self, params: *const Time, double_buffer_index: i32, _direct_process: ASIOBool) -> *const Time;
	fn get_sample_rate(&self) -> f64;
	fn get_driver_name(&self) -> &str;
	fn set_sample_rate(&mut self, sample_rate: f64) -> bool;
	fn start(&mut self);
	fn stop(&mut self);
}

pub struct ASIODevice<T> {	
	iasio: Option<IASIO>,
	#[allow(dead_code)]
	callbacks: Box<Callbacks>,
	pub driver_name: String,
	pub input_channels: Box<[InputChannel<T>]>,
	pub output_channels: Box<[OutputChannel<T>]>
}

// TODO: Really not sure if IASIO is sync safe?
unsafe impl<T> std::marker::Sync for ASIODevice<T> {
}

impl<T: 'static + Copy> ASIODevice<T> {
	pub fn set_active_device<'a>(clsid : com::CLSID) -> &'a mut dyn ASIODeviceType {
		unsafe {
			let mut dev = ASIODevice::<T>::new();
			dev.open(clsid);
			THE_DEVICE = Some(Box::new(dev));
			//THE_DEVICE.as_mut().unwrap()
			THE_DEVICE.unwrap().as_mut()
		}
	}

	fn new() -> ASIODevice<T> {
		ASIODevice {
			iasio: None,
			callbacks: Box::new(Callbacks {
				buffer_switch: cb_buffer_switch,
				sample_rate_did_change: cb_sample_rate_did_change,
				asio_message: cb_asio_message,
				buffer_switch_time_info: cb_buffer_switch_time_info
			}),
			driver_name: String::from("Null Device"),
			input_channels: Vec::<InputChannel<T>>::new().into_boxed_slice(),
			output_channels: Vec::<OutputChannel<T>>::new().into_boxed_slice()
		}
	}

	pub fn open(&mut self, clsid: com::CLSID) {
		let iasio = match create_device(&clsid) {
			Ok(value) => value,
			Err(hr) => panic!("Failed to create ASIO device: 0x{:x}", hr),
		};

		self.iasio = Some(iasio);

		let iasio_ref = self.iasio.as_ref().unwrap();
		self.driver_name = ASIODevice::<T>::get_driver_name(iasio_ref);	

		let pref_buffer_size = ASIODevice::<T>::get_buffer_size(iasio_ref);
		let (max_input_channels, max_output_channels) = ASIODevice::<T>::get_channel_count(iasio_ref);
		let num_input_channels = core::cmp::min(max_input_channels, 2);
		let num_output_channels = core::cmp::min(max_output_channels, 2);

		let buffer_infos = ASIODevice::<T>::create_buffers(iasio_ref, num_input_channels, num_output_channels, pref_buffer_size, &self.callbacks);

		let mut input_channels = Vec::<InputChannel<T>>::new();
		for index in 0..num_input_channels {
			let buffer_info = &buffer_infos[index as usize];
			input_channels.push(ASIODevice::<T>::get_input_channel(iasio_ref, index, buffer_info, pref_buffer_size));
		}
		self.input_channels = input_channels.into_boxed_slice();

		let mut output_channels = Vec::<OutputChannel<T>>::new();
		for index in 0..num_output_channels {
			let buffer_info = &buffer_infos[(num_input_channels + index) as usize];
			output_channels.push(ASIODevice::<T>::get_output_channel(iasio_ref, index, buffer_info, pref_buffer_size));
		}
		self.output_channels = output_channels.into_boxed_slice();
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
				ASIOBool::False => panic!("Driver initialization failed: {}", ASIODevice::<T>::get_error_message(&iasio)),
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

	fn get_channel_name(iasio: &IASIO, is_input: bool, id: i32) -> String {
		let asio_is_input = match is_input {
			true => ASIOBool::True,
			false => ASIOBool::False
		};

		let mut channel_info = ChannelInfo::new_for(asio_is_input, id);

		unsafe {
			if iasio.get_channel_info(&mut channel_info) != ASIOError::Ok {
				panic!("Failed to get channel description for input {}", id);
			}
		}

		String::from_utf8(channel_info.name.to_vec()).expect("Channel name is utf-8")
	}

	fn get_input_channel(iasio: &IASIO, id: i32, buffer_info : &BufferInfo, buffer_size: i32) -> InputChannel<T> {

		let buffer_a: *const () = buffer_info.buffers[0];
		let buffer_b: *const () = buffer_info.buffers[1];

		InputChannel::<T>::new(&ASIODevice::<T>::get_channel_name(iasio, true, id), buffer_a as *const T, buffer_b as *const T, buffer_size as usize)
	}

	fn get_output_channel(iasio: &IASIO, id: i32, buffer_info : &BufferInfo, buffer_size: i32) -> OutputChannel<T> {

		let buffer_a: *mut () = buffer_info.buffers[0];
		let buffer_b: *mut () = buffer_info.buffers[1];

		OutputChannel::<T>::new(&ASIODevice::<T>::get_channel_name(iasio, false, id), buffer_a as *mut T, buffer_b as *mut T, buffer_size as usize)
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

	fn get_input(&mut self, index: usize) -> &mut InputChannel<T> {
		// let input_channels = &self.input_channels.as_mut();
		// let boxed = input_channels[index].native_buffer;
		// let unboxed = &mut *boxed;

		// unboxed
		&mut self.input_channels[index]
	}

	fn get_output(&mut self, index: usize) -> &mut OutputChannel<T> {
		&mut self.output_channels[index]
	}
}

impl<T> ASIODeviceType for ASIODevice<T> {
	fn buffer_switch(&mut self, params: *const Time, double_buffer_index: i32, _direct_process: ASIOBool) -> *const Time {
		
		// The double_buffer_index indicates, 
		// - which output buffer the host should now start to fill
		// - which input buffer is filled with incoming data by the driver
		let write_second_half = double_buffer_index != 0;
		let read_second_half = double_buffer_index == 0;

		params
	}

	fn set_sample_rate(&mut self, sample_rate: f64) -> bool {
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

	fn get_sample_rate(&self) -> f64 {
		let iasio_ref = self.iasio.as_ref().unwrap();

		let mut effective_sample_rate = 0f64;
		unsafe{
			iasio_ref.get_sample_rate(&mut effective_sample_rate);
		}
		effective_sample_rate
	}

	fn get_driver_name(&self) -> &str {
		&self.driver_name
	}

	fn start(&mut self) {
		let iasio_ref = self.iasio.as_ref().unwrap();
		unsafe {
			iasio_ref.start();
		}
	}

	fn stop(&mut self) {
		let iasio_ref = self.iasio.as_ref().unwrap();
		unsafe {
			iasio_ref.stop();
		}
	}

}

pub static mut THE_DEVICE : Option<Box<dyn ASIODeviceType>> = None;

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