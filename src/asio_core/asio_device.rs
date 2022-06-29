use crate::asio_core::{ IASIO, Callbacks, ASIOBool, ASIOError, BufferInfo, ChannelInfo, Time };
use crate::asio_core::input_channel::InputChannel;
use crate::asio_core::output_channel::OutputChannel;

use super::output_channel;

pub trait ASIODeviceType {
	fn buffer_switch(&mut self, params: *const Time, double_buffer_index: i32, _direct_process: ASIOBool) -> *const Time;
	fn get_sample_rate(&self) -> f64;
	fn get_driver_name(&self) -> &str;
	fn set_sample_rate(&mut self, sample_rate: f64) -> bool;
	fn start(&mut self);
	fn stop(&mut self);
}

pub struct ASIODevice<T> {	
	iasio: IASIO,
	#[allow(dead_code)]
	callbacks: Box<Callbacks>,
	pub driver_name: String,
	pub input_channels: Box<[InputChannel<T>]>,
	pub output_channels: Box<[OutputChannel<T>]>
}

impl<T: 'static + Copy> ASIODevice<T> {

	pub fn new(iasio: IASIO, driver_name: String, num_input_channels: i32, num_output_channels: i32, 
		pref_buffer_size: i32, buffer_infos: Vec<BufferInfo>, callbacks: Callbacks) -> ASIODevice<T> {

		let mut input_channels = Vec::<InputChannel<T>>::new();
		for index in 0..num_input_channels {
			let buffer_info = &buffer_infos[index as usize];
			input_channels.push(ASIODevice::<T>::get_input_channel(&iasio, index, buffer_info, pref_buffer_size));
		}
		let input_channels = input_channels.into_boxed_slice();

		let mut output_channels = Vec::<OutputChannel<T>>::new();
		for index in 0..num_output_channels {
			let buffer_info = &buffer_infos[(num_input_channels + index) as usize];
			output_channels.push(ASIODevice::<T>::get_output_channel(&iasio, index, buffer_info, pref_buffer_size));
		}
		let output_channels = output_channels.into_boxed_slice();

		ASIODevice {
			iasio: iasio,
			callbacks: Box::new(callbacks),
			driver_name: driver_name,
			input_channels: input_channels,
			output_channels: output_channels
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

	fn get_input(&mut self, index: usize) -> &mut InputChannel<T> {
		&mut self.input_channels[index]
	}

	fn get_output(&mut self, index: usize) -> &mut OutputChannel<T> {
		&mut self.output_channels[index]
	}

	fn get_both_test(&mut self) -> (&mut InputChannel<T>, &mut OutputChannel<T>) {
		(&mut self.input_channels[0], &mut self.output_channels[0])
	}

}

impl<T: 'static + Copy> ASIODeviceType for ASIODevice<T> {
	fn buffer_switch(&mut self, params: *const Time, double_buffer_index: i32, _direct_process: ASIOBool) -> *const Time {
		
		// The double_buffer_index indicates, 
		// - which output buffer the host should now start to fill
		// - which input buffer is filled with incoming data by the driver
		// let input_channel = self.get_input(0);		
		// let output_channel = self.get_output(0);

		let (input_channel, output_channel) = self.get_both_test();

		input_channel.select_buffer(double_buffer_index);
		output_channel.select_buffer(double_buffer_index);
		output_channel.write(input_channel);

		params
	}

	fn set_sample_rate(&mut self, sample_rate: f64) -> bool {
		let iasio_ref = &self.iasio;

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
		let iasio_ref = &self.iasio;

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
		let iasio_ref = &self.iasio;
		unsafe {
			iasio_ref.start();
		}
	}

	fn stop(&mut self) {
		let iasio_ref = &self.iasio;
		unsafe {
			iasio_ref.stop();
		}
	}

}

