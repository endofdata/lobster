use std::sync::Once;
use windows::{
    core::{w, Interface, Result, GUID, HSTRING, PCWSTR},
    Graphics::SizeInt32,
    Win32::{
        Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, RECT, WPARAM},
        System::{LibraryLoader::GetModuleHandleW, WinRT::Composition::ICompositorDesktopInterop},
        UI::WindowsAndMessaging::{
            AdjustWindowRectEx, CreateWindowExW, DefWindowProcW, GetClientRect, GetWindowLongPtrW,
			LoadCursorW, MessageBoxW, PostQuitMessage, RegisterClassW, SetWindowLongPtrW, ShowWindow,
			CREATESTRUCTW, CW_USEDEFAULT, GWLP_USERDATA, IDC_ARROW, MB_ICONINFORMATION, SW_SHOW,
			WM_DESTROY, WM_LBUTTONDOWN, WM_NCCREATE, WNDCLASSW, WS_EX_NOREDIRECTIONBITMAP, WS_OVERLAPPEDWINDOW
        },
    },
    UI::Composition::{Compositor, Desktop::DesktopWindowTarget},
};
//use windows_numerics::Vector2;

use crate::{error::Error, vst_host::{host::Host, IEditController, IPlugView, VST_AUDIO_EFFECT_CLASS}};

static REGISTER_WINDOW_CLASS: Once = Once::new();
const WINDOW_CLASS_NAME: PCWSTR = w!("vsthost-rs.Window");

pub struct AppWindow {
    handle: HWND,
    //host: Host,
}

impl AppWindow {
    pub fn new(title: &str, width: u32, height: u32) -> Result<Box<Self>> {
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

        let window_ex_style = WS_EX_NOREDIRECTIONBITMAP;
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

        let mut result = Box::new(Self {
            handle: HWND::default(),
            //host,
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
                Some(result.as_mut() as *mut _ as _),
            )?
        };
        unsafe { _ = ShowWindow(window, SW_SHOW) };

        Ok(result)
    }

	#[allow(dead_code)]
    pub fn get_size(&self) -> Result<SizeInt32> {
        get_window_size(self.handle)
    }

    pub fn handle(&self) -> HWND {
        self.handle
    }

    pub fn create_window_target(
        &self,
        compositor: &Compositor,
        is_topmost: bool,
    ) -> Result<DesktopWindowTarget> {
        let compositor_desktop: ICompositorDesktopInterop = compositor.cast()?;
        unsafe { compositor_desktop.CreateDesktopWindowTarget(self.handle(), is_topmost) }
    }

    fn message_handler(&mut self, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        match message {
            WM_DESTROY => {
                unsafe { PostQuitMessage(0) };
                return LRESULT(0);
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
                if let Err(e) = vst_check() {
					unsafe { MessageBoxW(Some(self.handle), &HSTRING::from(e.to_string()), w!("VST check failed"), MB_ICONINFORMATION) };
				}
            }
            // WM_RBUTTONDOWN => {
            //     self.game.on_pointer_pressed(true, false).unwrap();
            // }
            _ => {}
        }
        unsafe { DefWindowProcW(self.handle, message, wparam, lparam) }
    }

    unsafe extern "system" fn wnd_proc(
        window: HWND,
        message: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
		unsafe {
			if message == WM_NCCREATE {
				let cs = lparam.0 as *const CREATESTRUCTW;
				let this = (*cs).lpCreateParams as *mut Self;
				(*this).handle = window;

				SetWindowLongPtrW(window, GWLP_USERDATA, this as _);
			} else {
				let this = GetWindowLongPtrW(window, GWLP_USERDATA) as *mut Self;

				if let Some(this) = this.as_mut() {
					return this.message_handler(message, wparam, lparam);
				}
			}
			DefWindowProcW(window, message, wparam, lparam)
		}
    }
}

fn get_window_size(window_handle: HWND) -> Result<SizeInt32> {
    unsafe {
        let mut rect = RECT::default();
        GetClientRect(window_handle, &mut rect)?;
        let width = rect.right - rect.left;
        let height = rect.bottom - rect.top;
        Ok(SizeInt32 {
            Width: width,
            Height: height,
        })
    }
}

#[allow(dead_code)]
fn get_mouse_position(lparam: LPARAM) -> (isize, isize) {
    let x = lparam.0 & 0xffff;
    let y = (lparam.0 >> 16) & 0xffff;
    (x, y)
}

fn vst_check() -> std::result::Result<(), crate::Error> {
	// Yamaha Steinberg USB ASIO
	let clsid = GUID {
		data1: 0xCB7F9FFD,
		data2: 0xA33B,
		data3: 0x48B2,
		data4: [0x8B, 0xC0, 0x43, 0x7D, 0x94, 0xF3, 0x71, 0x42],
	};

	let mut host = Host::new(&clsid, "Lobster")?;

	let library_path = "C:\\Program Files\\Common Files\\VST3\\Unfiltered Audio Indent.vst3";
	//let library_path = "C:\\Program Files\\Common Files\\VST3\\LVCMeter_x64.vst3";

	let vst_id = host.add_plugin_library(library_path)?;
	let lib = host.get_plugin_library(&vst_id)?;

	println!("Created VST:\n  Vendor: {:?}\n  URL: {:?}\n  EMail: {:?}\n  Flags: {:?}\n  Class Infos:",
		lib.get_vendor(), lib.get_url(), lib.get_email(), lib.get_flags());

	let mut audio_effect_id : Option<GUID> = None;

	for info in lib.get_class_infos() {
		println!("    {:?} {:?} {:?}: {:?} - {:?} [{:?}]", info.vendor, info.name, info.version, info.category, info.sub_categories, info.cid);
		if let Some(cat) = info.category {
			if cat == VST_AUDIO_EFFECT_CLASS {
				audio_effect_id = Some(info.cid)
			}
		}
	}

	if audio_effect_id.is_none() {
		eprintln!("No class of category '{}' was found.", VST_AUDIO_EFFECT_CLASS);
	}
	else {

		if let Ok(plugin) = lib.create_plugin(host.get_application(), &audio_effect_id) {
			println!("Created plugin");

			let edit_controller : IEditController = plugin.get_edit_controller()
				.or_else(|e| Err(Error::from_hresult("failed to get edit controller", e.code())))?;

			let parameter_count = unsafe { edit_controller.getParameterCount() };
			println!("  Plugin has {} parameter(s).", parameter_count);

			if let Some(plug_view) = unsafe {
				let raw_ptr = edit_controller.createView("editor".as_ptr());

				if raw_ptr != std::ptr::null() {
					//let raw_ptr = option.unwrap();
					let iface : IPlugView = windows_core::Interface::from_raw(raw_ptr as *mut std::ffi::c_void);
					//let iface = option.unwrap();
					let _test = iface.canResize().is_ok();
					Some(iface)
				}
				else {
					eprintln!("Cannot create 'editor' view. Method returned null.");
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
	}
	// drop host before uninitializing COM
	//drop(host);

	Ok(())
}
