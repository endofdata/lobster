use std::{cell::RefCell, sync::OnceLock};
use windows::{
    core::{Result, GUID},
    Win32::{
        Foundation::{E_FAIL, HWND, LPARAM, LRESULT, WPARAM},
        UI::WindowsAndMessaging::{
            WM_CREATE, WM_DESTROY, WM_LBUTTONDOWN, WS_EX_OVERLAPPEDWINDOW, WS_OVERLAPPEDWINDOW
        },
    },
};

use crate::{
	plug_frame::{PlugFrame, Resizable},
	vst_host::{
		host::Host, plugin::Plugin, thread_check::ThreadCheck,
		IPlugView, ViewRect, VST_AUDIO_EFFECT_CLASS
	},
	ui::{WndBase, WndClass, Boilerplate}
};

static WINDOW_CLASS: OnceLock<Result<u16>> = OnceLock::new();

pub struct AppWindow {
    handle: Option<HWND>,
	title: Option<String>,
	host: Host,
	vst_id: Option<String>,
	plugin: Option<Plugin>,
	resize_recursion_guard: RefCell<bool>
}

impl AppWindow {
    pub fn new(title: &str, width: u32, height: u32, host: Host) -> Result<Box<Self>> {

		let mut bp = Boilerplate::<AppWindow>::new();

		bp.register(&WINDOW_CLASS, "lobster.wndclass", None)?;

        let mut app_wnd = Box::new(Self {
            handle: None,
			title: Some(title.to_string()),
            host,
			vst_id: None,
			plugin: None,
			resize_recursion_guard: RefCell::new(false)
		});

		// WS_EX_NOREDIRECTIONBITMAP
		bp.create_window(&mut app_wnd, width, height, WS_OVERLAPPEDWINDOW, WS_EX_OVERLAPPEDWINDOW)?;

		app_wnd.show();

        Ok(app_wnd)
    }

	fn add_plugin(&mut self, library_path: &str) -> std::result::Result<String, crate::Error> {
		let vst_id = self.host.add_plugin_library(library_path)?;
		Ok(vst_id)
	}

	fn create_plugin(&self, vst_id: &str, thread_check: ThreadCheck, fx_id: Option<GUID>, category: Option<&str>) -> std::result::Result<Plugin, crate::Error> {
		let lib = self.host.get_plugin_library(&vst_id)?;
		let category = category.unwrap_or(VST_AUDIO_EFFECT_CLASS);
		let audio_effect_id = fx_id.or_else(|| {
			for info in lib.get_class_infos() {
				if let Some(cat) = info.category {
					if cat == category {
						return Some(info.cid)
					}
				}
			}
			None
		});

		if audio_effect_id.is_none() {
			Err(crate::Error::from_other("No class of category '{}' was found.", ))
		}
		else {
			let plugin = lib.create_plugin(self.host.get_application(), &audio_effect_id, thread_check)?;
			Ok(plugin)
		}
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
					self.create_plugin(vst_id, ThreadCheck::for_current_thread(), None, None)
						.and_then(|mut plugin| plugin.create_view(
							&PlugFrame::new(self).into(),
							&self.get_handle().expect("Window should have a valid handle"))
						.or_else(|e| Err(e.into()))
						.and_then(|_| {
							self.plugin = Some(plugin);
							Ok(())
					})).unwrap_or_else(|e| _ = self.show_error(&e));
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

impl Resizable for AppWindow {
	fn resize_view(&self, _view: &IPlugView, new_size: &ViewRect) -> Result<()> {
		if *self.resize_recursion_guard.borrow() == true {
			Ok(())
		}
		else {
			*self.resize_recursion_guard.borrow_mut() = true;

			let result = self.set_window_size(new_size.into());

			*self.resize_recursion_guard.borrow_mut() = false;

			result
		}
	}
}

impl Drop for AppWindow {
	fn drop(&mut self) {
		if let Some(plugin) = self.plugin.take() {
			drop(plugin);
		}
	}
}

