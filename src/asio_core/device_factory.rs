use crate::asio_core::{ IASIO, Callbacks, ASIOBool, ASIOError, ASIOSampleType, DriverInfo, BufferInfo, ChannelInfo, create_device };
use crate::asio_core::asio_device::{ASIODeviceType, ASIODevice};
use crate::asio_core::device_singleton::DeviceSingleton;

pub struct DeviceFactory {
}

impl DeviceFactory {
	pub fn create_device(clsid : com::CLSID) -> &'static mut dyn ASIODeviceType {
		DeviceSingleton::new(Box::new(DeviceFactory::open(clsid)));
		DeviceSingleton::get_device()
	}

	pub fn drop_device() {
		DeviceSingleton::drop()
	}

	fn open(clsid: com::CLSID) -> impl ASIODeviceType {
		let iasio = match create_device(&clsid) {
			Ok(value) => value,
			Err(hr) => panic!("Failed to create ASIO device: 0x{:x}", hr),
		};

		let driver_name = DeviceFactory::get_driver_name(&iasio);	
		let pref_buffer_size = DeviceFactory::get_buffer_size(&iasio);
		let (max_input_channels, max_output_channels) = DeviceFactory::get_channel_count(&iasio);
		let num_input_channels = core::cmp::min(max_input_channels, 2);
		let num_output_channels = core::cmp::min(max_output_channels, 2);
		let callbacks = Box::new(DeviceSingleton::init_callbacks());

		let buffer_infos = DeviceFactory::create_buffers(&iasio, num_input_channels, num_output_channels, pref_buffer_size, &callbacks);

		// TODO: Is it sufficient to peek the sample type from the first available output channel?
		let mut channel_info = ChannelInfo::new_for(ASIOBool::False, 0);

		unsafe {
			iasio.get_channel_info(&mut channel_info);
		}

		match channel_info.sample_type {
			ASIOSampleType::Int32LSB => ASIODevice::<i32>::new(iasio, driver_name, num_input_channels, num_output_channels, pref_buffer_size, buffer_infos, callbacks),
			_ => panic!("Unsupported sample type '{:?}'.", channel_info.sample_type)
		}
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
				ASIOBool::False => panic!("Driver initialization failed: {}", DeviceFactory::get_error_message(&iasio)),
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

}

