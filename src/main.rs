mod vst_host;
mod appwnd;
mod error;
mod frame;
mod pluginwnd;
mod ui;
mod os;

use error::Error;
use appwnd::AppWnd;

use windows::Win32::System::Com::COINIT_APARTMENTTHREADED;
#[rustfmt::skip]
use windows::{
	core::GUID
};

use crate::{ui::Vector2D, vst_host::host::Host};

// Yamaha Steinberg USB ASIO
const ASIO_DEVICE_CLSID : GUID = GUID {
	data1: 0xCB7F9FFD,
	data2: 0xA33B,
	data3: 0x48B2,
	data4: [0x8B, 0xC0, 0x43, 0x7D, 0x94, 0xF3, 0x71, 0x42],
};

const VST_LIBRARY_PATH : &str = "C:\\Program Files\\Common Files\\VST3\\Unfiltered Audio Indent.vst3";
//const VST_LIBRARY_PATH : &str = "C:\\Program Files\\Common Files\\VST3\\LVCMeter_x64.vst3";

fn main() -> std::result::Result<(), crate::Error> {
	os::co_initialize_ex(Some(COINIT_APARTMENTTHREADED))?;

	// scope to enforce cleanup before RoUninitialize
	{
		let host = Host::new(&ASIO_DEVICE_CLSID, "Lobster")?;
		let _window = AppWnd::new("VST Host", &Vector2D::new(1280, 800), host)?;

		ui::run_message_loop();

		println!("Shutting down");
	}

	os::co_uninitialize();

	Ok(())
}
