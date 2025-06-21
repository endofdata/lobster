mod vst_host;

use crate::vst_host::host::Host;
use windows::core::GUID;
use windows::Win32::System::Com::{CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED};

fn main() {
	let hr = unsafe {
		CoInitializeEx(None, COINIT_APARTMENTTHREADED)
	};

	if hr.is_err() {
		panic!("COM initialization failed with error code {:X}", hr.0);
	}


	// Yamaha Steinberg USB ASIO
	let clsid = GUID {
		data1: 0xCB7F9FFD,
		data2: 0xA33B,
		data3: 0x48B2,
		data4: [0x8B, 0xC0, 0x43, 0x7D, 0x94, 0xF3, 0x71, 0x42],
	};

	// println!("FIDString: {}", vst_host::as_fid_string(&clsid));

	let mut host = Host::new(&clsid).expect("Failed to create host.");

	let library_path = "C:\\Program Files\\Common Files\\VST3\\Unfiltered Audio Indent.vst3";

	let vst_id = match host.add_plugin(library_path) {
		Err(error) => panic!("Failed to add plugin '{}': {:?}", library_path, error),
		Ok(id) => id
	};

	// TODO: use vst for audio processing / creation
	let vst = host.get_audio_processor(&vst_id).expect("Failed to create audio processor.");

	let _test = host.get_component(&vst_id).expect("Failed to create audio component.");

	println!("Shutting down");

	// drop VSTs and host before uninitializing COM
	drop(vst);
	drop(host);

	unsafe {
		CoUninitialize();
	}
}


