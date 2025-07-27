#[rustfmt::skip]
use std::{cell::RefCell, rc::Rc, sync::OnceLock};

#[rustfmt::skip]
use windows::{
    core::Result,
	Win32::{
        Foundation::{E_FAIL, HWND, LRESULT},
		Graphics::Gdi::GRAY_BRUSH,
        UI::WindowsAndMessaging::{
            WS_EX_APPWINDOW, WS_EX_OVERLAPPEDWINDOW, WS_HSCROLL, WS_OVERLAPPEDWINDOW, WS_VSCROLL, CW_USEDEFAULT,
			CREATESTRUCTW
        },
    }
};

use crate::{
	pluginwnd::PluginWnd, ui::{self, Area, ListBox, Vector2D, WndBase, WndClass, WndClassImpl}, vst_host::host::Host
};

static WINDOW_CLASS: OnceLock<Result<u16>> = OnceLock::new();

pub struct AppWnd {
    handle: Option<HWND>,
	title: Option<String>,
	host: Rc<RefCell<Host>>,
	vst_id: Option<String>,
	plugin_wnd: Option<Rc<RefCell<PluginWnd>>>,
	fxlist: Option<Rc<RefCell<ListBox>>>
}

impl AppWnd {
    pub fn new(title: &str, size: &Vector2D, host: Host) -> Result<Rc<RefCell<Self>>> {

		let mut class_impl = WndClassImpl::<AppWnd>::new();

		class_impl.register_with_init(&WINDOW_CLASS, "lobster.wndclass", None, &|class| {
			class.hbrBackground = ui::get_stock_brush(GRAY_BRUSH);
			Ok(())
		})?;

        let app_wnd = Self {
            handle: None,
			title: Some(title.to_string()),
            host: Rc::new(RefCell::new(host)),
			vst_id: None,
			plugin_wnd: None,
			fxlist: None,
		};

		let rc = class_impl.create_window(app_wnd, &Area::new(CW_USEDEFAULT, CW_USEDEFAULT, size.get_x(), size.get_y()),
			WS_OVERLAPPEDWINDOW | WS_HSCROLL | WS_VSCROLL, WS_EX_OVERLAPPEDWINDOW | WS_EX_APPWINDOW, None, None)?;

		rc.borrow().show();

        Ok(rc)
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

 	fn on_destroy(&self) -> Option<LRESULT> {
		Self::post_quit_message(0);
		Some(LRESULT(0))
    }

 	fn on_create_mut(&mut self, _: &CREATESTRUCTW) -> Option<LRESULT> {
		match self.add_plugin(crate::VST_LIBRARY_PATH) {
			Ok(vst_id) => self.vst_id = Some(vst_id),
			Err(e) => {
				self.vst_id = None;
				_ = self.show_error(&e);
			}
		};
		Some(LRESULT(0))
	}

	fn on_left_button_up_mut(&mut self, modifiers: &ui::MouseModifiers, _position: &Vector2D) -> Option<LRESULT> {
		if modifiers.has(ui::MouseModifierFlags::Control) {
			if let Some(vst_id) = &self.vst_id {
				self.plugin_wnd = PluginWnd::new("Plugin", &Vector2D::default(), Rc::clone(&self.host), vst_id, self.get_handle().ok()).ok();
			};
			Some(LRESULT(0))
		}
		else {
			None
		}
	}
}

