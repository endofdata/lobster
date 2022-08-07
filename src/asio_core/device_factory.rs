use crate::asio_core::asio_device::{ASIODevice, ASIODeviceType};
use crate::asio_core::device_singleton::DeviceSingleton;
use crate::asio_core::Error;
use crate::asio_core::{
	create_device, ASIOBool, ASIOError, ASIOSampleType, BufferInfo, Callbacks, ChannelInfo,
	DriverInfo, IASIO,
};

pub struct DeviceFactory {}

impl DeviceFactory {
	pub fn create_device(
		clsid: com::CLSID,
		process: fn(input: Vec<Vec<f64>>, outputs: &mut [Vec<f64>]),
	) -> Result<&'static mut dyn ASIODeviceType, Error> {
		match DeviceFactory::open(clsid, process) {
			Ok(device) => {
				DeviceSingleton::new(Box::new(device));
				Ok(DeviceSingleton::get_device())
			}
			Err(e) => Err(e),
		}
	}

	pub fn drop_device() {
		DeviceSingleton::drop()
	}

	fn open(
		clsid: com::CLSID,
		process: fn(input: Vec<Vec<f64>>, outputs: &mut [Vec<f64>]),
	) -> Result<impl ASIODeviceType, Error> {
		match create_device(&clsid) {
			Ok(iasio) => {
				match DeviceFactory::init_device(&iasio) {
					Ok(()) => {
						let driver_name = DeviceFactory::get_driver_name(&iasio)?;
						let pref_buffer_size = DeviceFactory::get_buffer_size(&iasio)?;
						let (max_input_channels, max_output_channels) =
							DeviceFactory::get_channel_count(&iasio)?;
						let num_input_channels = core::cmp::min(max_input_channels, 2);
						let num_output_channels = core::cmp::min(max_output_channels, 2);
						let callbacks = Box::new(DeviceSingleton::init_callbacks());

						let buffer_infos = DeviceFactory::create_buffers(
							&iasio,
							num_input_channels,
							num_output_channels,
							pref_buffer_size,
							&callbacks,
						)?;

						// TODO: Is it sufficient to peek the sample type from the first available output channel?
						let mut channel_info = ChannelInfo::new_for(ASIOBool::False, 0);

						unsafe {
							iasio.get_channel_info(&mut channel_info);
						}

						match channel_info.sample_type {
							ASIOSampleType::Int32LSB => Ok(ASIODevice::<i32>::new(
								iasio,
								driver_name,
								num_input_channels,
								num_output_channels,
								pref_buffer_size,
								buffer_infos,
								callbacks,
								process,
							)),
							_ => Err(Error::from_other(&format!(
								"Unsupported sample type '{:?}'.",
								channel_info.sample_type
							))),
						}
					}
					Err(err) => Err(err),
				}
			}
			Err(hr) => Err(Error::from_hresult("Failed to create ASIO device", hr)),
		}
	}

	fn init_device(iasio: &IASIO) -> Result<(), Error> {
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
				ASIOBool::False => Err(Error::from_other(&format!(
					"Driver initialization failed: {}",
					DeviceFactory::get_error_message(&iasio)
				))),
				ASIOBool::True => Ok(()),
			}
		}
	}

	fn get_driver_name(iasio: &IASIO) -> Result<String, Error> {
		unsafe {
			let mut buffer = vec![0u8; 128];
			let ptr = buffer.as_mut_ptr();
			iasio.get_driver_name(ptr);

			let trimmed: Vec<u8> = buffer.iter().take_while(|c| **c != 0u8).cloned().collect();
			return Ok(String::from_utf8(trimmed).expect("Driver name is valid UTF-8"));
		}
	}

	fn get_buffer_size(iasio: &IASIO) -> Result<i32, Error> {
		let mut min_buffer_size = 0i32;
		let mut max_buffer_size = 0i32;
		let mut pref_buffer_size = 0i32;
		let mut granularity = 0i32;
		unsafe {
			match iasio.get_buffer_size(
				&mut min_buffer_size,
				&mut max_buffer_size,
				&mut pref_buffer_size,
				&mut granularity,
			) {
				ASIOError::Ok => Ok(pref_buffer_size),
				other => Err(Error::from_asio("Failed to get buffer size", other)),
			}
		}
	}

	fn get_channel_count(iasio: &IASIO) -> Result<(i32, i32), Error> {
		let mut max_input_channels: i32 = 0;
		let mut max_output_channels: i32 = 0;

		unsafe {
			match iasio.get_channels(&mut max_input_channels, &mut max_output_channels) {
				ASIOError::Ok => Ok((max_input_channels, max_output_channels)),
				other => Err(Error::from_asio("Failed to get channels", other)),
			}
		}
	}

	fn create_buffers(
		iasio: &IASIO,
		num_input_channels: i32,
		num_output_channels: i32,
		pref_buffer_size: i32,
		callbacks: &Callbacks,
	) -> Result<Vec<BufferInfo>, Error> {
		let mut buffer_infos =
			Vec::<BufferInfo>::with_capacity((num_input_channels + num_output_channels) as usize);

		for id in 0..num_input_channels {
			buffer_infos.push(BufferInfo {
				channel_num: id,
				is_input: ASIOBool::True,
				buffers: [core::ptr::null_mut::<()>(); 2],
			});
		}

		for id in 0..num_output_channels {
			buffer_infos.push(BufferInfo {
				channel_num: id,
				is_input: ASIOBool::False,
				buffers: [core::ptr::null_mut::<()>(); 2],
			});
		}

		unsafe {
			let result = iasio.create_buffers(
				buffer_infos.as_mut_ptr(),
				buffer_infos.len() as i32,
				pref_buffer_size,
				callbacks,
			);
			if result != ASIOError::Ok {
				return Err(Error::from_asio("Failed to create buffers: {:?}", result));
			};
		}
		Ok(buffer_infos)
	}

	fn get_error_message(iasio: &IASIO) -> String {
		let mut buffer = vec![0u8; 256];
		let ptr = buffer.as_mut_ptr();

		unsafe {
			iasio.get_error_message(ptr);
		}
		let trimmed: Vec<u8> = buffer.iter().take_while(|c| **c != 0u8).cloned().collect();

		String::from_utf8(trimmed).expect("Error message is valid UTF-8")
	}
}
