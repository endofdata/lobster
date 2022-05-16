mod asio_core;

use std::thread;
use std::time::Duration;

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


		//println!("Driver name: '{}'", name);

		let sample_rate = 48000.0f64;


		// If we needed to make a void* from some value:
		//let mut enable = 1; // ASIOBool::True;
		//let void_ptr = core::ptr::addr_of_mut!(enable) as *mut ()
		// result = device.future(FutureSelector::EnableTimeCodeRead, core::ptr::null_mut::<()>());
		// match  result {
		// 	ASIOError::Ok => println!("Enabled time code read"),
		// 	ASIOError::NotPresent => println!("Time code read not supported"),
		// 	_ => panic!("Failed to enable time code read {:?}", result)
		// };

		let device = asio_core::asio_device::ASIODevice::<i32>::new(clsid);
		
		device.start();

		println!("ASIO Device started");

		thread::sleep(Duration::from_secs(2));

		device.stop();
		
		println!("ASIO device stopped");
	}

	unsafe {
		com::sys::CoUninitialize();
	}
}
