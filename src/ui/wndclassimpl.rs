use std::{
	marker::PhantomData, sync::OnceLock
};

#[rustfmt::skip]
use windows::{
	core::{Error, Result, HSTRING, PCWSTR},
	Win32::{
		Foundation::{E_FAIL, E_INVALIDARG, HINSTANCE, HWND, LPARAM, LRESULT, WPARAM},
		System::LibraryLoader::GetModuleHandleW,
		UI::WindowsAndMessaging::{
			CreateWindowExW, DefWindowProcW, GetWindowLongPtrW, LoadCursorW, RegisterClassW, SetWindowLongPtrW,
			CREATESTRUCTW, CW_USEDEFAULT, GWLP_USERDATA, IDC_ARROW, WINDOW_EX_STYLE, WINDOW_STYLE, WM_NCCREATE, WNDCLASSW, HMENU
		}
	}
};

use super::{WndBase, WndClass};
use crate::os::StrConv;

pub struct WndClassImpl<W> {
	phantom: PhantomData<W>,
	atom: Option<u16>,
	instance: Option<HINSTANCE>,
}

impl<W> WndClassImpl<W> {
	pub fn new() -> Self {
		Self {
			phantom: PhantomData,
			atom: None,
			instance: None
		}
	}
}

impl<W: WndBase> WndClass for WndClassImpl<W> {
	type WndType = W;

	fn register(&mut self, once: &'static OnceLock<Result<u16>>, class_name: &str, instance: Option<HINSTANCE>) -> Result<u16> {
		once.get_or_init(|| {
			let mut class_name_w = [0u16; 258];
			StrConv::str_to_slice_w(class_name, &mut class_name_w, true);

			let class = WNDCLASSW {
				hCursor: unsafe { LoadCursorW(None, IDC_ARROW).ok().expect("Windows should provide the IDC_ARROW cursor") },
				hInstance: instance.unwrap_or_else(|| unsafe { GetModuleHandleW(None) }.expect("GetModuleHandleW(None) should not fail").into() ),
				lpszClassName: PCWSTR(class_name_w.as_ptr()),
				lpfnWndProc: Some(Self::wnd_proc),
				..Default::default()
			};
			match unsafe { RegisterClassW(&class) } {
				0  => Err(Error::from_win32()),
				atom => {
					self.atom = Some(atom);
					self.instance = Some(class.hInstance);
					Ok(atom)
				}
			}
		}).clone()
	}

	fn create_window(&self, outer: &mut Self::WndType, width: u32, height: u32, style: WINDOW_STYLE, ex_style: WINDOW_EX_STYLE, parent: Option<HWND>, menu: Option<HMENU>) -> Result<()> {
		self.atom.ok_or_else(|| Error::from_hresult(E_FAIL))
		.and_then(|atom| {
			super::adjust_window_size(width, height, style, ex_style)
			.or_else(|_| Err(Error::from_hresult(E_INVALIDARG)))
			.and_then(|(adjusted_width, adjusted_height)| {
				let ptr_fake: *const u16 = std::ptr::without_provenance(atom as usize);
				let class_name = PCWSTR::from_raw(ptr_fake);
				let title = outer.get_title().unwrap_or("");
				unsafe {
					CreateWindowExW(
						ex_style,
						class_name,
						&HSTRING::from(title),
						style,
						CW_USEDEFAULT,
						CW_USEDEFAULT,
						adjusted_width as i32,
						adjusted_height as i32,
						parent,
						menu,
						self.instance,
						Some(outer as *mut W as _),
					)
				}
				.and_then(|window| {
					outer.set_handle(Some(window));
					Ok(())
				})
			})
		})
	}

	unsafe extern "system" fn wnd_proc(handle: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
		unsafe {
			if message == WM_NCCREATE {
				let create_struct = lparam.0 as *const CREATESTRUCTW;
				let outer = (*create_struct).lpCreateParams as *mut Self::WndType;

				(*outer).set_handle(Some(handle));
				SetWindowLongPtrW(handle, GWLP_USERDATA, outer.addr().try_into()
					.expect("Window address should fit into isize"));
			} else {
				let outer = GetWindowLongPtrW(handle, GWLP_USERDATA) as *mut Self::WndType;
				if outer != std::ptr::null_mut() {
					return (*outer).on_message(message, wparam, lparam);
				}
			}
			DefWindowProcW(handle, message, wparam, lparam)
		}
	}
}

