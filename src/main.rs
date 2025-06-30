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

		let mut host = Host::new(&clsid, "Lobster")?;

		let library_path = "C:\\Program Files\\Common Files\\VST3\\Unfiltered Audio Indent.vst3";

		let vst_id = host.add_plugin_library(library_path)?;
		let vst = host.get_plugin_library(&vst_id)?;

		println!("Created VST:\n  Vendor: {}\n  URL: {}\n  EMail: {}\n  Flags: {:?}\n  Class Infos:",
			vst.get_vendor(), vst.get_url(), vst.get_email(), vst.get_flags());

		for info in vst.get_class_infos() {
			println!("    {} {} {}: {} - {} [{:?}]", info.vendor, info.name, info.version, info.category, info.sub_categories, info.cid);
		}

		if let Ok(plugin) = vst.create_plugin(host.get_application()) {
			println!("Created plugin");

			let edit_controller : IEditController = plugin.get_edit_controller()
				.or_else(|e| Err(Error::from_hresult("failed to get edit controller", e.code())))?;

			let parameter_count = unsafe { edit_controller.getParameterCount() };
			println!("  Plugin has {} parameter(s).", parameter_count);

			if let Some(plug_view) = unsafe {
				if let Some(fred) = edit_controller.createView("editor".as_ptr()).as_mut() {
					Some(fred.clone())
				}
				else {
					None
				}
			} {
				let can_resize = unsafe  { plug_view.canResize() }.is_ok();
				println!("  PlugView can resize: {}", can_resize);
			}

			let audio_processor = plugin.create_audio_processor()
				.or_else(|e| Err(Error::from_hresult("failed to create audio processor", e.code())))?;

			let sample_size = std::mem::size_of::<f32>() as i32;
			if unsafe { audio_processor.canProcessSampleSize(sample_size).is_err() } {
				println!("  Plugin cannot process samples of size {} byte(s).", sample_size);
			}
			else {
				println!("  Plugin can process samples of size {} byte(s).", sample_size);
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


