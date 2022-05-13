mod asio_core;

use std::thread;
use std::time::Duration;
use asio_core::*;

fn main() {

	let hr = unsafe {
		com::sys::CoInitializeEx(
			core::ptr::null_mut::<core::ffi::c_void>(),
			com::sys::COINIT_APARTMENTTHREADED,
		)
	};

	if !com::sys::FAILED(hr) {
		// Yamaha Steinberg USB ASIO
		let clsid = com::CLSID {
			data1: 0xCB7F9FFD,
			data2: 0xA33B,
			data3: 0x48B2,
			data4: [0x8B, 0xC0, 0x43, 0x7D, 0x94, 0xF3, 0x71, 0x42],
		};

		let device = match create_device(&clsid) {
			Ok(value) => value,
			Err(hr) => panic!("Failed to create ASIO device: 0x{:x}", hr),
		};

		unsafe {
			//let mut name: [u8; 128] = [0; 128];
			//let ptr = name.as_mut_ptr();

			let mut driver_info = DriverInfo {
				asio_version: 2,
				driver_version: 0,
				name: [0; 32],
				error_message: [0; 124],
				sys_ref: core::ptr::null::<()>(),
			};

			let driver_info_ptr: *mut DriverInfo = &mut driver_info;

			let succ = device.init(driver_info_ptr as *mut ());

			match succ {
				ASIOBool::False => println!("{} WTF?!? This do not happened, whizzco!", get_error_message(&device)),
				ASIOBool::True =>	{
					let mut buffer = vec![0u8; 128];
					let ptr = buffer.as_mut_ptr();
					device.get_driver_name(ptr);

					let trimmed : Vec<u8> = buffer.iter().take_while(|c| **c != 0u8).cloned().collect();
					let name = String::from_utf8(trimmed)
						.expect("Driver name is valid UTF-8");
					
					println!("Driver name: '{}'", name);

					let sample_rate = 48000.0f64;

					match device.can_sample_rate(sample_rate) {
						ASIOError::Ok => println!("Sample rate {} is promised", sample_rate),
						_ => panic!("Desired sample rate is not supported.")
					};

					match device.set_sample_rate(sample_rate) {
						ASIOError::Ok => println!("Sample rate {} is set", sample_rate),
						_ => panic!("Cannot set desired sample rate")
					};

					let mut effective_sample_rate = 0f64;

					device.get_sample_rate(&mut effective_sample_rate);
					
					match effective_sample_rate == sample_rate
					{
						true => println!("Sample rate {} is confirmed", sample_rate),
						_ => panic!("Desired sample rate is not confirmed")
					};

					let mut num_input_channels: i32 = 0;
					let mut num_output_channels: i32 = 0;

					match device.get_channels(&mut num_input_channels, &mut num_output_channels) {
						ASIOError::Ok => println!("Channels: in({}) out({})", num_input_channels, num_output_channels),
						_ => panic!("Failed to get channels information")
					};

					let mut min_buffer_size = 0i32;
					let mut max_buffer_size = 0i32;
					let mut pref_buffer_size = 0i32;
					let mut granularity = 0i32;

					match device.get_buffer_size(&mut min_buffer_size, &mut max_buffer_size, &mut pref_buffer_size, &mut granularity) {
						ASIOError::Ok => println!("Buffer size: min({}) max({}) pref({}) granularity({})", min_buffer_size, max_buffer_size, pref_buffer_size, granularity),
						_ => panic!("Failed to get buffer size information")
					};


					const MAX_CLOCK_SOURCES : usize = 4;
					let mut clock_source_count = MAX_CLOCK_SOURCES as i32;
					let mut clock_sources = [ClockSource::new(); MAX_CLOCK_SOURCES];
					
					match device.get_clock_sources(clock_sources.as_mut_ptr(), &mut clock_source_count) {
						ASIOError::Ok => {
							println!("Clock sources: {}", clock_source_count);
							for clock_source in clock_sources.iter().take(clock_source_count as usize) {
								println!("{:#?}", &clock_source);
							}
						},
						_ => panic!("Failed to get clock source information")
					}
					
					let callbacks = Callbacks {
						buffer_switch: cb_buffer_switch,
						sample_rate_did_change: cb_sample_rate_did_change,
						asio_message: cb_asio_message,
						buffer_switch_time_info: cb_buffer_switch_time_info
					};

					let mut buffer_infos: [BufferInfo; 4] = [
						BufferInfo {
							channel_num: 0,
							is_input: 1,
							buffers: [core::ptr::null_mut::<()>(); 2]
						},
						BufferInfo {
							channel_num: 1,
							is_input: 1,
							buffers: [core::ptr::null_mut::<()>(); 2]
						},
						BufferInfo {
							channel_num: 0,
							is_input: 0,
							buffers: [core::ptr::null_mut::<()>(); 2]
						},
						BufferInfo {
							channel_num: 1,
							is_input: 0,
							buffers: [core::ptr::null_mut::<()>(); 2]
						},
					];

					let mut result = device.create_buffers(buffer_infos.as_mut_ptr(), buffer_infos.len() as i32, pref_buffer_size, &callbacks);
					match result {
						ASIOError::Ok => println!("Created two input and two output buffers of {} bytes", pref_buffer_size),
						_ => panic!("Failed to create ASIO buffers: {:?}", result)
					};

					let selectors = [
						FutureSelector::CanInputMonitor,
						FutureSelector::CanTimeInfo,
						FutureSelector::CanTimeCode,
						FutureSelector::CanTransport,
						FutureSelector::CanInputGain,
						FutureSelector::CanInputMeter,
						FutureSelector::CanOutputGain,
						FutureSelector::CanOutputMeter
					];

					for sel in selectors.iter() {
						result = device.future(FutureSelector::EnableTimeCodeRead, core::ptr::null_mut::<()>());
						match result {
							ASIOError::Ok => println!("'{:?}': yes", sel),
							ASIOError::NotPresent => println!("{:?}: no", sel),
							_ => println!("Unexpected answer on querying '{:?}': {:?}", sel, result)
						};
					}
					

					// If we needed to make a void* from some value:
					//let mut enable = 1; // ASIOBool::True;
					//let void_ptr = core::ptr::addr_of_mut!(enable) as *mut ()
					result = device.future(FutureSelector::EnableTimeCodeRead, core::ptr::null_mut::<()>());
					match  result {
						ASIOError::Ok => println!("Enabled time code read"),
						ASIOError::NotPresent => println!("Time code read not supported"),
						_ => panic!("Failed to enable time code read {:?}", result)
					};
					
					device.start();

					println!("ASIO Device started");

					thread::sleep(Duration::from_secs(2));
		
					device.stop();
					
					println!("ASIO device stopped");

					println!("Received {} ASIO buffer switches", BUFFER_COUNT);

					// This gives us bullshit :-)
					println!("ASIO Time: {:#?}", ASIO_TIME);
				}
			}
		
		}
	}

	unsafe {
		com::sys::CoUninitialize();
	}
}

fn get_error_message(device: &IASIO) -> String {
	let mut buffer = vec![0u8; 256];
	let ptr = buffer.as_mut_ptr();

	unsafe {
		device.get_error_message(ptr);
	}
	let trimmed : Vec<u8> = buffer.iter().take_while(|c| **c != 0u8).cloned().collect();

	String::from_utf8(trimmed)
		.expect("Error message is valid UTF-8")
}


static mut BUFFER_COUNT: u32 = 0;

static mut ASIO_TIME: Time = Time::new();

extern "C" fn cb_buffer_switch(double_buffer_index: i32, direct_process: ASIOBool) {
	cb_buffer_switch_time_info(core::ptr::null::<Time>(), double_buffer_index, direct_process);
}

extern "C" fn cb_buffer_switch_time_info(_params: *const Time, _double_buffer_index: i32, _direct_process: ASIOBool) -> *const Time {
	unsafe {
		BUFFER_COUNT = BUFFER_COUNT + 1;
		if _params != core::ptr::null::<_>() {
			ASIO_TIME.clone_from(&*_params);
		}
		_params
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




