mod asio_core;
mod vst_host;

use std::thread;
use std::time::Duration;
use vst_host::plugin_factory::{PluginFactory, PluginLibrary};

fn main() {
	let hr = unsafe {
		com::sys::CoInitializeEx(
			core::ptr::null_mut::<core::ffi::c_void>(),
			com::sys::COINIT_APARTMENTTHREADED,
		)
	};

	if !com::sys::FAILED(hr) {

		let library_path = "C:\\Program Files\\Common Files\\VST3\\Unfiltered Audio Indent.vst3";

		load_vst(library_path);

		// Yamaha Steinberg USB ASIO
		let clsid = com::CLSID {
			data1: 0xCB7F9FFD,
			data2: 0xA33B,
			data3: 0x48B2,
			data4: [0x8B, 0xC0, 0x43, 0x7D, 0x94, 0xF3, 0x71, 0x42],
		};

		match asio_core::device_factory::DeviceFactory::create_device(clsid, process_buffers) {
			Err(error) => println!("Failed to create ASIO device: {:?}", error),
			Ok(device) => {
				println!("Created ASIO device '{}'", device.get_driver_name());

				device.set_sample_rate(48000.0f64);

				println!("ASIO device starting");
				device.start();
				println!("ASIO Device started");

				thread::sleep(Duration::from_secs(2));

				println!("ASIO device stopping");
				device.stop();
				println!("ASIO device stopped");

				asio_core::device_factory::DeviceFactory::drop_device();
			}
		}		
	}

	println!("Shutting down");
	
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

fn show_factory_info(factory: &PluginFactory) {
	println!(
		"Vendor: '{}', E-Mail: '{}', Url: '{}'",
		factory.vendor, factory.email, factory.url
	);

	let class_count = factory.count_classes();
	if class_count == 0 {
		println!("No classes defined");
	} else {
		for c in 0..class_count {
			match factory.get_class_info(c) {
				Some(class_info) => println!("Class {}: {:#?}", c, class_info),
				None => println!("Class {}: No information", c),
			}
		}
	}
}

fn load_vst(library_path: &str) {

	// the lifetime of the Library must exceed the lifetime of all interfaces
	match PluginLibrary::load(library_path) {
		Ok(vst) => match vst.get_factory() {
			Ok(factory) => {
				println!("Loaded VST from '{}'", library_path);
				show_factory_info(&factory);
			}
			Err(error) => println!("Failed to create factory: {:?}", error),
		},
		Err(error) => println!("Failed to load plugin: {:?}", error),
	};
}
