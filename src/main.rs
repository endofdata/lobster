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

		let device =
			asio_core::device_factory::DeviceFactory::create_device(clsid, process_buffers);

		println!("Created ASIO device '{}'", device.get_driver_name());

		device.set_sample_rate(48000.0f64);

		println!("ASIO device starting");
		device.start();
		println!("ASIO Device started");

		thread::sleep(Duration::from_secs(10));

		println!("ASIO device stopping");
		device.stop();
		println!("ASIO device stopped");

		asio_core::device_factory::DeviceFactory::drop_device();
	}

	unsafe {
		com::sys::CoUninitialize();
	}
}

fn process_buffers(input: Vec<Vec<f64>>, outputs: &mut [Vec<f64>]) {
	let ins = input.len() as i32;
	let outs = outputs.len() as i32;

	if ins >= 1 {
		if outs == 2 {
			for o in 0..outs {
				let mut proc = input[0].iter().map(|s| *s * 1.0);
				let dst = outputs[o as usize].iter_mut();
				for target in dst {
					*target = proc.next().expect("Not enough input data");
				}
			}
		}
	}
}
