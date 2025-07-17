use std::{any::Any, cell::RefCell, sync::Once};
use windows::{
    core::{implement, w, Interface, Result, GUID, HRESULT, HSTRING, PCWSTR},
    Graphics::SizeInt32,
    Win32::{
        Foundation::{E_FAIL, HINSTANCE, HWND, LPARAM, LRESULT, RECT, S_OK, WPARAM},
        System::{LibraryLoader::GetModuleHandleW, WinRT::Composition::ICompositorDesktopInterop, Threading::GetCurrentThreadId},
        UI::WindowsAndMessaging::{
            AdjustWindowRectEx, CreateWindowExW, DefWindowProcW, GetClientRect, GetWindowInfo, GetWindowLongPtrW, LoadCursorW, MessageBoxW, PostQuitMessage, RegisterClassW, SendMessageW, SetWindowLongPtrW, SetWindowPos, ShowWindow, CREATESTRUCTW, CW_USEDEFAULT, GWLP_USERDATA, HWND_TOP, IDC_ARROW, MB_ICONWARNING, SWP_NOACTIVATE, SWP_NOCOPYBITS, SWP_NOMOVE, SWP_NOSIZE, SWP_SHOWWINDOW, SW_SHOW, WINDOWINFO, WM_CREATE, WM_DESTROY, WM_LBUTTONDOWN, WM_NCCREATE, WNDCLASSW, WS_EX_NOREDIRECTIONBITMAP, WS_EX_OVERLAPPEDWINDOW, WS_OVERLAPPEDWINDOW
        },
    },
    UI::Composition::{Compositor, Desktop::DesktopWindowTarget},
};
use windows_core::{AsImpl, ComObject, ComObjectInner, ComObjectInterface, IUnknown, InterfaceRef};

//use windows_numerics::Vector2;

use crate::{plug_frame::PlugFrame, vst_host::{
	host::Host, plugin::Plugin, thread_check::ThreadCheck, Error, IPlugFrame, IPlugFrame_Impl, IPlugView, ViewRect, VST_AUDIO_EFFECT_CLASS
}};

static REGISTER_WINDOW_CLASS: Once = Once::new();
const WINDOW_CLASS_NAME: PCWSTR = w!("vsthost-rs.Window");

pub struct AppWindow {
    handle: HWND,
	host: Host,
	vst_id: Option<String>,
	plugin: Option<Plugin>,
	resize_recursion_guard: RefCell<bool>
}

impl AppWindow {
    pub fn new(title: &str, width: u32, height: u32, host: Host) -> Result<Box<Self>> {
        let instance = unsafe { GetModuleHandleW(None)? };
        REGISTER_WINDOW_CLASS.call_once(|| {
            let class = WNDCLASSW {
                hCursor: unsafe { LoadCursorW(None, IDC_ARROW).ok().unwrap() },
                hInstance: instance.into(),
                lpszClassName: WINDOW_CLASS_NAME,
                lpfnWndProc: Some(Self::wnd_proc),
                ..Default::default()
            };
            assert_ne!(unsafe { RegisterClassW(&class) }, 0);
        });

        let window_ex_style = WS_EX_OVERLAPPEDWINDOW; // | WS_EX_NOREDIRECTIONBITMAP;
        let window_style = WS_OVERLAPPEDWINDOW;

        let (adjusted_width, adjusted_height) = {
            let mut rect = RECT {
                left: 0,
                top: 0,
                right: width as i32,
                bottom: height as i32,
            };
            unsafe {
                AdjustWindowRectEx(&mut rect, window_style, false, window_ex_style)?;
            }
            (rect.right - rect.left, rect.bottom - rect.top)
        };

        let mut app_wnd = Box::new(Self {
            handle: HWND::default(),
            host,
			vst_id: None,
			plugin: None,
			resize_recursion_guard: RefCell::new(false)
		});

        let hinstance: HINSTANCE = instance.into();

        let window = unsafe {
            CreateWindowExW(
                window_ex_style,
                WINDOW_CLASS_NAME,
                &HSTRING::from(title),
                window_style,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                adjusted_width,
                adjusted_height,
                None,
                None,
                Some(hinstance),
                Some(app_wnd.as_mut() as *mut AppWindow as _),
            )?
        };
        unsafe { _ = ShowWindow(window, SW_SHOW) };

        Ok(app_wnd)
    }

	pub fn resize_view(&self, view: &IPlugView, new_size: &ViewRect) -> Result<()> {
		if *self.resize_recursion_guard.borrow() == true {
			Ok(())
		}
		else {
			*self.resize_recursion_guard.borrow_mut() = true;

			let mut window_info = WINDOWINFO::default();
			let mut client_rect = RECT { left: 0, top: 0, right: new_size.get_width(), bottom: new_size.get_height()};

			let result = unsafe {
				GetWindowInfo (self.handle, &mut window_info)
					.and_then(|_| AdjustWindowRectEx (&mut client_rect, window_info.dwStyle, false, window_info.dwExStyle))
					.and_then(|_| SetWindowPos (
						self.handle, Some(HWND_TOP), 0, 0,
						client_rect.right - client_rect.left,
						client_rect.bottom - client_rect.top,
						SWP_NOMOVE | SWP_NOCOPYBITS | SWP_NOACTIVATE))
			};

			*self.resize_recursion_guard.borrow_mut() = false;

			result
		}
	}

    pub fn create_window_target(
        &self,
        compositor: &Compositor,
        is_topmost: bool,
    ) -> Result<DesktopWindowTarget> {
        let compositor_desktop: ICompositorDesktopInterop = compositor.cast()?;
        unsafe { compositor_desktop.CreateDesktopWindowTarget(self.get_handle(), is_topmost) }
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


    fn on_message(&mut self, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        match message {
			WM_CREATE => {
				match self.add_plugin(crate::VST_LIBRARY_PATH) {
					Ok(vst_id) => self.vst_id = Some(vst_id),
					Err(e) => {
						self.vst_id = None;
						self.show_error(e);
					}
				};
			}
            // WM_MOUSEMOVE => {
            //     let (x, y) = get_mouse_position(lparam);
            //     let point = Vector2 {
            //         X: x as f32,
            //         Y: y as f32,
            //     };
            //     self.game.on_pointer_moved(&point).unwrap();
            // }
            // WM_SIZE | WM_SIZING => {
            //     let new_size = self.size().unwrap();
            //     let new_size = Vector2 {
            //         X: new_size.Width as f32,
            //         Y: new_size.Height as f32,
            //     };
            //     self.game.on_parent_size_changed(&new_size).unwrap();
            // }
            WM_LBUTTONDOWN => {
				if let Some(vst_id) = &self.vst_id {
					self.create_plugin(vst_id, ThreadCheck::for_current_thread(), None, None)
						.and_then(|mut plugin| plugin.create_view(&PlugFrame::new(self).into(), &self.get_handle())
							.or_else(|e| Err(e.into()))
							.and_then(|_| {
								self.plugin = Some(plugin);
								Ok(())
					})).unwrap_or_else(|e| self.show_error(e));
				}
            }
			WM_DESTROY => {
				unsafe { PostQuitMessage(0) };
				return LRESULT(0);
            }
			_ => {}
        }
        unsafe { DefWindowProcW(self.handle, message, wparam, lparam) }
    }

	fn show_error(&self, error: crate::Error) {
		unsafe { MessageBoxW(Some(self.handle), &HSTRING::from(error.to_string()), w!("Lobster"), MB_ICONWARNING) };
	}

    pub fn get_handle(&self) -> HWND {
        self.handle
    }

	#[allow(dead_code)]
	fn get_window_size(&self) -> Result<SizeInt32> {
		unsafe {
			let mut rect = RECT::default();

			GetClientRect(self.handle, &mut rect)?;

			Ok(SizeInt32 {
				Width: rect.right - rect.left,
				Height: rect.bottom - rect.top,
			})
		}
	}

	#[allow(dead_code)]
	fn get_mouse_position(lparam: LPARAM) -> (isize, isize) {
		(lparam.0 & 0xffff, (lparam.0 >> 16) & 0xffff)
	}

    unsafe extern "system" fn wnd_proc(handle: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
		unsafe {
			if message == WM_NCCREATE {
				let cs = lparam.0 as *const CREATESTRUCTW;
				let app_wnd = (*cs).lpCreateParams as *mut Self;

				(*app_wnd).handle = handle;
				SetWindowLongPtrW(handle, GWLP_USERDATA, app_wnd as _);
			} else {
				let this = GetWindowLongPtrW(handle, GWLP_USERDATA) as *mut Self;

				if let Some(this) = this.as_mut() {
					return this.on_message(message, wparam, lparam);
				}
			}
			DefWindowProcW(handle, message, wparam, lparam)
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



