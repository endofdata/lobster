#[rustfmt::skip]
use std::{cell::RefCell, rc::Rc, sync::OnceLock};

use windows::Win32::Graphics::Gdi::GRAY_BRUSH;
#[rustfmt::skip]
use windows::{
    core::Result,
	Win32::{
        Foundation::{E_FAIL, HWND, LPARAM, LRESULT, WPARAM},
        UI::WindowsAndMessaging::{
            WM_CREATE, WM_DESTROY, WM_LBUTTONDOWN, WS_EX_APPWINDOW, WS_EX_OVERLAPPEDWINDOW, WS_HSCROLL, WS_OVERLAPPEDWINDOW, WS_VSCROLL
        },
    }
};

use crate::{
	pluginwnd::PluginWnd, ui::{WndClassImpl, WndBase, WndClass}, ui, vst_host::host::Host
};

static WINDOW_CLASS: OnceLock<Result<u16>> = OnceLock::new();

pub struct AppWnd {
    handle: Option<HWND>,
	title: Option<String>,
	host: Rc<RefCell<Host>>,
	vst_id: Option<String>,
	plugin_wnd: Option<Box<PluginWnd>>
}

impl AppWnd {
    pub fn new(title: &str, width: u32, height: u32, host: Host) -> Result<Box<Self>> {

		let mut class_impl = WndClassImpl::<AppWnd>::new();

		class_impl.register_with_init(&WINDOW_CLASS, "lobster.wndclass", None, &|class| {
			class.hbrBackground = ui::get_stock_brush(GRAY_BRUSH);
			Ok(())
		})?;

        let mut app_wnd = Box::new(Self {
            handle: None,
			title: Some(title.to_string()),
            host: Rc::new(RefCell::new(host)),
			vst_id: None,
			plugin_wnd: None
		});

		// WS_EX_OVERLAPPEDWINDOW
		class_impl.create_window(&mut app_wnd, width, height,
			WS_OVERLAPPEDWINDOW | WS_HSCROLL | WS_VSCROLL, WS_EX_OVERLAPPEDWINDOW | WS_EX_APPWINDOW, None, None)?;

		app_wnd.show();

        Ok(app_wnd)
    }

	fn add_plugin(&mut self, library_path: &str) -> std::result::Result<String, crate::Error> {
		let vst_id = self.host.borrow_mut().add_plugin_library(library_path)?;
		Ok(vst_id)
	}

}

impl WndBase for AppWnd {
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

