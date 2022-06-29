use crate::asio_core::{ Callbacks, ASIOBool, MessageSelector, Time };
use crate::asio_core::asio_device::{ ASIODeviceType };
use std::sync::{Arc, Once};
use std::{mem::MaybeUninit};

pub struct DeviceSingleton {
	pub value: Arc<Box<dyn ASIODeviceType>>
}

static mut THE_DEVICE : MaybeUninit<DeviceSingleton> = MaybeUninit::uninit();

impl DeviceSingleton {

	/// Constructs a singleton that provides sync'ed access to the box'ed value
	pub fn new(value: Box<dyn ASIODeviceType>) -> &'static DeviceSingleton {
		static ONCE : Once = Once::new();

		unsafe {
			ONCE.call_once(|| {
				let singleton = DeviceSingleton {
					value: Arc::new(value)
				};
				THE_DEVICE.write(singleton);
			});

			THE_DEVICE.assume_init_ref()
		}
	}

	/// Gets a mutable reference for the singleton's (unboxed) value
	pub fn get_device<'a>() -> &'a mut dyn ASIODeviceType {
		let singleton;
		unsafe {
			singleton = THE_DEVICE.assume_init_mut();
		}
		let boxed = Arc::get_mut(&mut singleton.value).expect("Cannot access singleton value");
		let mutref = Box::as_mut(boxed);

		mutref
	}

	pub fn drop() {
		unsafe {
			THE_DEVICE.assume_init_drop();
		}	
	}

	pub fn init_callbacks() -> Callbacks {
		Callbacks {
			buffer_switch: DeviceSingleton::cb_buffer_switch,
			sample_rate_did_change: DeviceSingleton::cb_sample_rate_did_change,
			asio_message: DeviceSingleton::cb_asio_message,
			buffer_switch_time_info: DeviceSingleton::cb_buffer_switch_time_info
		}
	}

	extern "C" fn cb_buffer_switch(double_buffer_index: i32, direct_process: ASIOBool) {
		DeviceSingleton::cb_buffer_switch_time_info(core::ptr::null::<Time>(), double_buffer_index, direct_process);
	}

	extern "C" fn cb_buffer_switch_time_info(params: *const Time, double_buffer_index: i32, direct_process: ASIOBool) -> *const Time {
		DeviceSingleton::get_device().buffer_switch(params, double_buffer_index, direct_process)
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
}
