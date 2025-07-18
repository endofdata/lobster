use std::{
	any::Any, cell::RefCell, marker::PhantomData, os::windows::io::AsHandle, sync::{Once, OnceLock}
};

use windows::{
	core::{Error, Result}, Foundation::Size, Graphics::SizeInt32, Win32::{
		Foundation::{GetLastError, E_FAIL, E_INVALIDARG, HINSTANCE, HMODULE, HWND, LPARAM, LRESULT, RECT, WPARAM}, System::LibraryLoader::GetModuleHandleW, UI::WindowsAndMessaging::{AdjustWindowRectEx, CreateWindowExW, DefWindowProcW, GetClientRect, GetWindowInfo, GetWindowLongPtrW, LoadCursorW, MessageBoxW, PostQuitMessage, RegisterClassW, SendMessageW, SetWindowLongPtrW, SetWindowPos, ShowWindow, CREATESTRUCTW, CW_USEDEFAULT, GWLP_USERDATA, HWND_TOP, IDC_ARROW, MB_ICONWARNING, SWP_NOACTIVATE, SWP_NOCOPYBITS, SWP_NOMOVE, SWP_NOSIZE, SWP_SHOWWINDOW, SW_SHOW, WINDOWINFO, WINDOW_EX_STYLE, WINDOW_STYLE, WM_CREATE, WM_DESTROY, WM_LBUTTONDOWN, WM_NCCREATE, WNDCLASSW, WS_EX_NOREDIRECTIONBITMAP, WS_EX_OVERLAPPEDWINDOW, WS_OVERLAPPEDWINDOW}
	}
};
use windows_core::{w, HSTRING, PCWSTR};
use crate::os::StrConv;

pub trait WndBase {
	fn show_window(&self);
	fn set_handle(&mut self, handle: Option<HWND>);
	fn on_message(&mut self, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT;
}

pub trait WndClass {
	type WndType: WndBase;

	fn register(&mut self, once: &'static OnceLock<Result<u16>>, class_name: &str, instance: Option<HINSTANCE>) -> Result<u16>;
	fn create_window(&self, outer: &mut Self::WndType, title: &str, width: u32, height: u32, style: WINDOW_STYLE, ex_style: WINDOW_EX_STYLE) -> Result<()>;
	unsafe extern "system" fn wnd_proc(handle: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT;
}

pub struct Boilerplate<W> {
	phantom: PhantomData<W>,
	atom: Option<u16>,
	instance: Option<HINSTANCE>,
}

impl<W> Boilerplate<W> {
	pub fn new() -> Self {
		Self {
			phantom: PhantomData,
			atom: None,
			instance: None
		}
	}

	pub fn adjust_window_size(width: u32, height: u32, style: WINDOW_STYLE, ex_style: WINDOW_EX_STYLE) -> Result<(u32, u32)> {
		let mut rect = RECT {
			left: 0,
			top: 0,
			right: width as i32,
			bottom: height as i32,
		};
		unsafe {
			AdjustWindowRectEx(&mut rect, style, false, ex_style)?;
		}
		(rect.right - rect.left).try_into()
		.and_then(|width|
			(rect.bottom - rect.top).try_into()
			.and_then(|height|
				Ok((width, height))))
		.or_else(|_| Err(Error::from_hresult(E_INVALIDARG)))
	}
}


impl<W: WndBase> WndClass for Boilerplate<W> {
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

	fn create_window(&self, outer: &mut Self::WndType, title: &str, width: u32, height: u32, style: WINDOW_STYLE, ex_style: WINDOW_EX_STYLE) -> Result<()> {
		self.atom.ok_or_else(|| Error::from_hresult(E_FAIL))
		.and_then(|atom| {
			Self::adjust_window_size(width, height, style, ex_style)
			.or_else(|_| Err(Error::from_hresult(E_INVALIDARG)))
			.and_then(|(adjusted_width, adjusted_height)| {
				let ptr_fake: *const u16 = std::ptr::without_provenance(atom as usize);
				let class_name = PCWSTR::from_raw(ptr_fake);
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
						None,
						None,
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
				let app_wnd = (*create_struct).lpCreateParams as *mut Self::WndType;

				(*app_wnd).set_handle(Some(handle));
				SetWindowLongPtrW(handle, GWLP_USERDATA, app_wnd.addr().try_into()
					.expect("Window address should fit into isize"));
			} else {
				let app_wnd = GetWindowLongPtrW(handle, GWLP_USERDATA) as *mut Self::WndType;
				if app_wnd != std::ptr::null_mut() {
					return (*app_wnd).on_message(message, wparam, lparam);
				}
			}
			DefWindowProcW(handle, message, wparam, lparam)
		}
	}
}
