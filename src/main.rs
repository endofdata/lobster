mod vst_host;
mod appwnd;
mod interop;
mod error;

use error::Error;
use appwnd::AppWindow;
use interop::{
	create_dispatcher_queue_controller_for_current_thread,
	shutdown_dispatcher_queue_controller_and_wait
	//shutdown_dispatcher_queue_controller_and_exit
};
use windows::{
    Win32::{
        System::WinRT::{
			RoInitialize, RoUninitialize, RO_INIT_SINGLETHREADED
		},
        UI::WindowsAndMessaging::{
			DispatchMessageW, GetMessageW, TranslateMessage, MSG
		},
    },
    UI::Composition::Compositor,
};
use windows_numerics::Vector2;

fn main() -> std::result::Result<(), crate::Error> {
	unsafe {
		RoInitialize(RO_INIT_SINGLETHREADED)
			.or_else(|e| Err(Error::from_windows("Runtime initialization failed", e)))?;
	};
	// scope to enforce cleanup before RoUninitialize
	{
		let controller = create_dispatcher_queue_controller_for_current_thread()?;

		let compositor = Compositor::new()
			.or_else(|e| Err(Error::from_windows("Failed to create Compositor instance", e)))?;

		let root = compositor.CreateContainerVisual()
			.or_else(|e| Err(Error::from_windows("Cannot create root container visual", e)))?;

		root.SetRelativeSizeAdjustment(Vector2::new(1.0, 1.0))?;

		let window = AppWindow::new("VST Host", 800, 600)?;
		let target = window.create_window_target(&compositor, false)?;
		target.SetRoot(&root)?;

		let mut message = MSG::default();

		unsafe {
			while GetMessageW(&mut message, None, 0, 0).into() {
				_ = TranslateMessage(&message);
				DispatchMessageW(&message);
			}
		}

		println!("Shutting down");

		let _exit_code = shutdown_dispatcher_queue_controller_and_wait(&controller, 0)
			.or_else(|e| Err(Error::from_windows("Dispatcher queue shutdown failed", e)))?;

		//shutdown_dispatcher_queue_controller_and_exit(&controller, message.wParam.0 as i32);
	}
	unsafe {
		RoUninitialize();
	}
	Ok(())
}
