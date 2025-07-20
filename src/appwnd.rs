#[rustfmt::skip]
use std::{cell::RefCell, rc::Rc, sync::OnceLock};
use windows::{
    core::Result,
	System::DispatcherQueueController,
	Win32::{
        Foundation::{E_FAIL, HWND, LPARAM, LRESULT, WPARAM},
        UI::WindowsAndMessaging::{
            WM_CREATE, WM_DESTROY, WM_LBUTTONDOWN, WS_EX_APPWINDOW, WS_EX_NOREDIRECTIONBITMAP, WS_HSCROLL, WS_OVERLAPPEDWINDOW, WS_VSCROLL
        },
    }
};

use crate::{
	pluginwnd::PluginWnd, ui::{Boilerplate, WndBase, WndClass, Composable}, vst_host::host::Host
};

static WINDOW_CLASS: OnceLock<Result<u16>> = OnceLock::new();

pub struct AppWindow {
    handle: Option<HWND>,
	controller: Option<DispatcherQueueController>,
	title: Option<String>,
	host: Rc<RefCell<Host>>,
	vst_id: Option<String>,
	plugin_wnd: Option<Box<PluginWnd>>
}

impl AppWindow {
    pub fn new(title: &str, width: u32, height: u32, host: Host) -> Result<Box<Self>> {

		let mut bp = Boilerplate::<AppWindow>::new();

		bp.register(&WINDOW_CLASS, "lobster.wndclass", None)?;

        let mut app_wnd = Box::new(Self {
            handle: None,
			controller: None,
			title: Some(title.to_string()),
            host: Rc::new(RefCell::new(host)),
			vst_id: None,
			plugin_wnd: None
		});

		// WS_EX_OVERLAPPEDWINDOW
		bp.create_window(&mut app_wnd, width, height, WS_OVERLAPPEDWINDOW | WS_HSCROLL | WS_VSCROLL, WS_EX_NOREDIRECTIONBITMAP | WS_EX_APPWINDOW, None, None)?;

		app_wnd.create_controller(app_wnd.get_handle().unwrap_or_default(), true)?;
		app_wnd.show();

        Ok(app_wnd)
    }

	fn add_plugin(&mut self, library_path: &str) -> std::result::Result<String, crate::Error> {
		let vst_id = self.host.borrow_mut().add_plugin_library(library_path)?;
		Ok(vst_id)
	}

}

impl WndBase for AppWindow {
	fn set_handle(&mut self, handle: Option<HWND>) {
		self.handle = handle;
	}

	fn get_handle(&self) -> Result<HWND> {
		self.handle.ok_or(windows::core::Error::from_hresult(E_FAIL))
	}

	fn get_title(&self) -> Result<&str> {
		self.title.as_deref().ok_or(windows::core::Error::from_hresult(E_FAIL))
	}

 	fn on_message(&mut self, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        match message {
			WM_CREATE => {
				match self.add_plugin(crate::VST_LIBRARY_PATH) {
					Ok(vst_id) => self.vst_id = Some(vst_id),
					Err(e) => {
						self.vst_id = None;
						_ = self.show_error(&e);
					}
				};
			}
            WM_LBUTTONDOWN => {
				if let Some(vst_id) = &self.vst_id {
					self.plugin_wnd = PluginWnd::new("Plugin", 0, 0, Rc::clone(&self.host), vst_id, self.get_handle().ok()).ok();
				};
            }
			WM_DESTROY => {
				Self::post_quit_message(0);
				return LRESULT(0);
            }
			_ => {}
        }
		self.def_window_proc(message, wparam, lparam)
    }
}

impl Composable for AppWindow {
	fn set_controller(&mut self, controller: Option<DispatcherQueueController>) {
		self.controller = controller;
	}

	fn get_controller(&self) -> Result<&DispatcherQueueController> {
		self.controller.as_ref().ok_or(windows::core::Error::from_hresult(E_FAIL))
	}
}


impl Drop for AppWindow {
	fn drop(&mut self) {
		_ = self.shutdown_controller(0);
	}
}

