use std::{
	marker::PhantomData, sync::OnceLock
};

#[rustfmt::skip]
use windows::{
	core::{Interface, Error, Result, HSTRING, PCWSTR},
	Win32::{
		Foundation::{E_FAIL, E_INVALIDARG, HINSTANCE, HWND, LPARAM, LRESULT, RECT, WPARAM},
		System::{LibraryLoader::GetModuleHandleW, WinRT::Composition::ICompositorDesktopInterop},
		UI::WindowsAndMessaging::{
			AdjustWindowRectEx, CreateWindowExW, DefWindowProcW, GetClientRect, GetWindowInfo,
			GetWindowLongPtrW, GetWindowTextLengthW, GetWindowTextW, LoadCursorW, MessageBoxW,
			PostQuitMessage, RegisterClassW, SetWindowLongPtrW, SetWindowPos, ShowWindow,
			CREATESTRUCTW, CW_USEDEFAULT, GWLP_USERDATA, HWND_TOP, IDC_ARROW, MB_ICONWARNING,
			MESSAGEBOX_RESULT, MESSAGEBOX_STYLE, SWP_NOACTIVATE, SWP_NOCOPYBITS, SWP_NOMOVE,
			SW_SHOW, WINDOWINFO, WINDOW_EX_STYLE, WINDOW_STYLE, WM_NCCREATE, WNDCLASSW
		}
	},
	UI::Composition::{Compositor, Desktop::DesktopWindowTarget},
	Graphics::SizeInt32
};

use crate::{os::StrConv, vst_host::ViewRect};

impl Into<SizeInt32> for &ViewRect{
	fn into(self) -> SizeInt32 {
		SizeInt32 {
			Width: self.get_width(),
			Height: self.get_height()
		}
	}
}

pub trait WndBase {
	fn set_handle(&mut self, handle: Option<HWND>);
	fn get_handle(&self) -> Result<HWND>;
	fn on_message(&mut self, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT;

	fn show(&self) {
		if let Ok(handle) = self.get_handle() {
			unsafe { _ = ShowWindow(handle, SW_SHOW) };
		}
	}

	fn get_info(&self) -> Result<WINDOWINFO> {
		let handle = self.get_handle()?;
		let mut info = WINDOWINFO::default();
		unsafe { GetWindowInfo (handle, &mut info) }
		.and_then(|_| Ok(info))
	}

	fn get_title(&self) -> Result<&str>;

	#[allow(dead_code)]
	fn copy_title(&self) -> Result<String> {
		self.get_handle()
		.and_then(|handle| {
			unsafe {
				let length = GetWindowTextLengthW(handle) as usize;
				let mut buffer = Vec::<u16>::with_capacity(length + 1);
				GetWindowTextW(handle, &mut buffer);
				String::from_utf16(&buffer)
				.or_else(|_| Err(Error::from_hresult(E_FAIL)))
			}
		})
	}

	fn show_error<E: std::error::Error>(&self, error: &E) -> Result<MESSAGEBOX_RESULT> {
		self.show_error_with_title_and_style(error, None, None)
	}

	#[allow(dead_code)]
	fn show_error_with_title<E: std::error::Error>(&self, error: &E, title: &str) -> Result<MESSAGEBOX_RESULT> {
		self.show_error_with_title_and_style(error, Some(title), None)
	}

	#[allow(dead_code)]
	fn show_error_with_style<E: std::error::Error>(&self, error: &E, style: MESSAGEBOX_STYLE) -> Result<MESSAGEBOX_RESULT> {
		self.show_error_with_title_and_style(error, None, Some(style))
	}

	fn show_error_with_title_and_style<E: std::error::Error>(&self, error: &E, title: Option<&str>, style: Option<MESSAGEBOX_STYLE>) -> Result<MESSAGEBOX_RESULT> {
		let handle = self.get_handle()?;
		let title = title.and_then(|t| Some(HSTRING::from(t)))
		.unwrap_or_else(|| HSTRING::from(self.get_title().as_deref().unwrap_or_else(|_| "Error")));
		let style = style.unwrap_or(MB_ICONWARNING);
		Ok(unsafe { MessageBoxW(Some(handle), &HSTRING::from(error.to_string()), &title, style) })
	}

	#[allow(dead_code)]
	fn get_window_size(&self) -> Result<SizeInt32> {
		let handle = self.get_handle()?;
		unsafe {
			let mut rect = RECT::default();

			GetClientRect(handle, &mut rect)?;

			Ok(SizeInt32 {
				Width: rect.right - rect.left,
				Height: rect.bottom - rect.top,
			})
		}
	}

	fn set_window_size(&self, new_size: SizeInt32) -> Result<()> {
		let mut client_rect = RECT { left: 0, top: 0, right: new_size.Width, bottom: new_size.Height};
		self.adjust_window_rect(&mut client_rect).and_then(|_| {
		self.get_handle().and_then(|handle|
		unsafe { SetWindowPos (
			handle, Some(HWND_TOP), 0, 0,
			client_rect.right - client_rect.left,
			client_rect.bottom - client_rect.top,
			SWP_NOMOVE | SWP_NOCOPYBITS | SWP_NOACTIVATE) })
		})
	}

	fn adjust_window_rect(&self, client_rect: &mut RECT) -> Result<()> {
		let info = self.get_info()?;
		unsafe { AdjustWindowRectEx (client_rect, info.dwStyle, false, info.dwExStyle) }
	}

	fn def_window_proc(&self, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
		unsafe { DefWindowProcW(self.get_handle().unwrap_or_default(), message, wparam, lparam) }
	}

	fn post_quit_message(exit_code: i32) {
		unsafe { PostQuitMessage(exit_code) };
	}

	#[allow(dead_code)]
	fn as_xy(lparam: LPARAM) -> (isize, isize) {
		(lparam.0 & 0xffff, (lparam.0 >> 16) & 0xffff)
	}
}

pub trait Composable {
	#[allow(dead_code)]
	fn create_window_target(
		&self,
		compositor: &Compositor,
		is_topmost: bool,
	) -> Result<DesktopWindowTarget>;
}

impl<W: WndBase> Composable for W {
	fn create_window_target(
		&self,
		compositor: &Compositor,
		is_topmost: bool,
	) -> Result<DesktopWindowTarget> {
		let compositor_desktop: ICompositorDesktopInterop = compositor.cast()?;
		unsafe { compositor_desktop.CreateDesktopWindowTarget(self.get_handle().unwrap_or_default(), is_topmost) }
	}
}

pub trait WndClass {
	type WndType: WndBase;

	fn register(&mut self, once: &'static OnceLock<Result<u16>>, class_name: &str, instance: Option<HINSTANCE>) -> Result<u16>;
	fn create_window(&self, outer: &mut Self::WndType, width: u32, height: u32, style: WINDOW_STYLE, ex_style: WINDOW_EX_STYLE) -> Result<()>;
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

	fn create_window(&self, outer: &mut Self::WndType, width: u32, height: u32, style: WINDOW_STYLE, ex_style: WINDOW_EX_STYLE) -> Result<()> {
		self.atom.ok_or_else(|| Error::from_hresult(E_FAIL))
		.and_then(|atom| {
			Self::adjust_window_size(width, height, style, ex_style)
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

