use std::{cell::RefCell, rc::Rc, sync::OnceLock};

use windows::{
	core::Result, System::DispatcherQueueController, Win32::{Foundation::{E_FAIL, HWND, LPARAM, LRESULT, WPARAM}, UI::WindowsAndMessaging::{WM_CREATE, WM_DESTROY, WS_BORDER, WS_CAPTION, WS_CHILD, WS_EX_TOOLWINDOW, WS_SYSMENU}}
};
use windows_core::GUID;

use crate::{frame::{Frame, Resizable}, ui::{Boilerplate, WndBase, WndClass}, vst_host::{host::Host, plugin::Plugin, thread_check::ThreadCheck, IPlugView, ViewRect, VST_AUDIO_EFFECT_CLASS}};

static WINDOW_CLASS: OnceLock<Result<u16>> = OnceLock::new();

pub struct PluginWnd {
    handle: Option<HWND>,
	title: Option<String>,
	host: Rc<RefCell<Host>>,
	vst_id: String,
	plugin: Option<Plugin>,
	resize_recursion_guard: RefCell<bool>,
}

impl PluginWnd {
    pub fn new(title: &str, width: u32, height: u32, host: Rc<RefCell<Host>>, vst_id: &str, parent: Option<HWND>) -> Result<Box<Self>> {

		let mut bp = Boilerplate::<PluginWnd>::new();

		bp.register(&WINDOW_CLASS, "plugin.wndclass", None)?;

        let mut plugin_wnd = Box::new(Self {
            handle: None,
			title: Some(title.to_string()),
            host,
			vst_id: vst_id.to_string(),
			plugin: None,
			resize_recursion_guard: RefCell::new(false)
		});

		// WS_EX_NOREDIRECTIONBITMAP
		bp.create_window(&mut plugin_wnd, width, height, WS_CHILD | WS_BORDER | WS_CAPTION | WS_SYSMENU, WS_EX_TOOLWINDOW, parent, None)?;

		plugin_wnd.show();

        Ok(plugin_wnd)
    }

	fn create_plugin(&self, vst_id: &str, thread_check: ThreadCheck, fx_id: Option<GUID>, category: Option<&str>) -> std::result::Result<Plugin, crate::Error> {
		let host = self.host.borrow();
		let lib = host.get_plugin_library(&vst_id)?;
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
			let plugin = lib.create_plugin(self.host.borrow().get_application(), &audio_effect_id, thread_check)?;
			Ok(plugin)
		}
	}
}

impl WndBase for PluginWnd {
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
				self.create_plugin(&self.vst_id, ThreadCheck::for_current_thread(), None, None)
					.and_then(|mut plugin| plugin.create_view(
						&Frame::new(self).into(),
						&self.get_handle().expect("Window should have a valid handle"))
					.or_else(|e| Err(e.into()))
					.and_then(|_| {
						self.plugin = Some(plugin);
						Ok(())
				})).unwrap_or_else(|e| _ = self.show_error(&e));
            }
			WM_DESTROY => {
				return LRESULT(0);
            }
			_ => {}
        }
		self.def_window_proc(message, wparam, lparam)
    }
}

impl Resizable for PluginWnd {
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

impl Drop for PluginWnd {
	fn drop(&mut self) {
		if let Some(plugin) = self.plugin.take() {
			drop(plugin);
		}
	}
}

