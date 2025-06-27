mod vst_host;
mod error;

use crate::vst_host::host::Host;
use crate::error::Error;
use crate::vst_host::IEditController;
use windows::core::GUID;
use windows::Win32::System::Com::{CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED};

fn main() -> Result<(), Error> {
	let hr = unsafe {
		CoInitializeEx(None, COINIT_APARTMENTTHREADED)
	};

	if hr.is_err() {
		Err(Error::from_hresult("COM initialization failed", hr))
	}
	else {
		// Yamaha Steinberg USB ASIO
		let clsid = GUID {
			data1: 0xCB7F9FFD,
			data2: 0xA33B,
			data3: 0x48B2,
			data4: [0x8B, 0xC0, 0x43, 0x7D, 0x94, 0xF3, 0x71, 0x42],
		};

		println!("as_fid_tring:   {}", vst_host::as_fid_string(&clsid));
		println!("guid.to_string: {:?}", clsid);

		let mut host = Host::new(&clsid)?;

		let library_path = "C:\\Program Files\\Common Files\\VST3\\Unfiltered Audio Indent.vst3";

		let vst_id = host.add_plugin_library(library_path)?;

		// TODO: use vst for audio processing / creation
		//let vst = host.get_audio_processor(&vst_id)?;
		//let _test = host.get_component(&vst_id)?;

		if let Ok(cls_infos) = host.get_class_infos(&vst_id) {
			for info in cls_infos {
				println!("{} {} {}: {} - {} [{:?}]", info.vendor, info.name, info.version, info.category, info.sub_categories, info.cid);
			}
		}

		if let Ok(plugin) = host.get_plugin(&vst_id) {
			println!("Created plugin.");

			let edit_controller : IEditController = plugin.get_edit_controller()
				.or_else(|e| Err(Error::from_hresult("failed to get edit controller", e.code())))?;

			let parameter_count = unsafe { edit_controller.getParameterCount() };

			println!("Plugin has {} parameter(s).", parameter_count);

			let audio_processor = plugin.create_audio_processor()
				.or_else(|e| Err(Error::from_hresult("failed to create audio processor", e.code())))?;

			let sample_size = std::mem::size_of::<f32>() as i32;
			if unsafe { audio_processor.canProcessSampleSize(sample_size).is_err() } {
				println!("Plugin cannot process samples of size {} byte(s).", sample_size);
			}
			else {
				println!("Plugin can process samples of size {} byte(s).", sample_size);
			}
		}

		println!("Shutting down");

		// drop VSTs and host before uninitializing COM
		//drop(vst);
		drop(host);

		unsafe {
			CoUninitialize();
		}
		Ok(())
	}
}


