mod vst_host;
mod appwnd;
mod error;
mod frame;
mod pluginwnd;
mod ui;
mod os;

use error::Error;
use appwnd::AppWindow;

use windows::{
    Win32::{
        System::WinRT::{
			RoInitialize, RoUninitialize, RO_INIT_SINGLETHREADED
		},
        UI::WindowsAndMessaging::{
			DispatchMessageW, GetMessageW, TranslateMessage, MSG
		},
    }
};
use windows_core::GUID;

use crate::vst_host::host::Host;

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
	unsafe {
		RoInitialize(RO_INIT_SINGLETHREADED)
			.or_else(|e| Err(Error::from_windows("Runtime initialization failed", e)))?;
	};
	// scope to enforce cleanup before RoUninitialize
	{


		let host = Host::new(&ASIO_DEVICE_CLSID, "Lobster")?;
		let _window = AppWindow::new("VST Host", 800, 600, host)?;

		let mut message = MSG::default();

		unsafe {
			while GetMessageW(&mut message, None, 0, 0).into() {
				_ = TranslateMessage(&message);
				DispatchMessageW(&message);
			}
		}

		println!("Shutting down");


	}
	unsafe {
		RoUninitialize();
	}
	Ok(())
}
