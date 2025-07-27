use std::{
	cell::RefCell, marker::PhantomData, ops::{Deref, DerefMut}, rc::Rc, sync::OnceLock
};

#[rustfmt::skip]
use windows::{
	core::{Error, Result, HSTRING, PCWSTR},
	Win32::{
		Foundation::{
			E_FAIL, E_INVALIDARG,
			HINSTANCE, HWND, LPARAM, LRESULT, WPARAM
		},
		System::LibraryLoader::GetModuleHandleW,
		UI::WindowsAndMessaging::{
			CreateWindowExW, DefWindowProcW, GetWindowLongPtrW, LoadCursorW, RegisterClassW, SetWindowLongPtrW,
			CW_USEDEFAULT, GWLP_USERDATA, IDC_ARROW, WM_NCCREATE, WM_DESTROY, WM_CLOSE, WM_CREATE, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_SHOWWINDOW,
			CREATESTRUCTW, WINDOW_EX_STYLE, WINDOW_STYLE, WNDCLASSW, HMENU
		}
	}
};

use super::{WndBase, WndClass};
use crate::{os::StrConv, ui::{modifiers::Position, MouseModifiers}};

pub struct WndClassImpl<W> {
	phantom: PhantomData<W>,
	atom: Option<u16>,
	instance: Option<HINSTANCE>,
}

impl<W: WndBase> WndClassImpl<W> {
	pub fn new() -> Self {
		Self {
			phantom: PhantomData,
			atom: None,
			instance: None
		}
	}

	fn on_message_mut(outer_mut: &mut W, message: u32, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
		match message {
			WM_CREATE => outer_mut.on_create_mut(&unsafe { *(lparam.0 as *const CREATESTRUCTW)}),
			WM_SHOWWINDOW => {
				if wparam.0 != 0 {
					outer_mut.on_show_mut(lparam.0)
				}
				else {
					outer_mut.on_hide_mut(lparam.0)
				}
			}
			WM_LBUTTONDOWN => outer_mut.on_left_button_down_mut(&MouseModifiers::from_wparam(wparam), &Position::from_lparam(lparam)),
			WM_LBUTTONUP => outer_mut.on_left_button_up_mut(&MouseModifiers::from_wparam(wparam), &Position::from_lparam(lparam)),
			WM_CLOSE => outer_mut.on_close_mut(),
			WM_DESTROY => outer_mut.on_destroy_mut(),
			_ => None
		}
	}

	fn on_message(outer: &W, message: u32, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
		match message {
			WM_CREATE => outer.on_create(&unsafe { *(lparam.0 as *const CREATESTRUCTW)}),
			WM_SHOWWINDOW => {
				if wparam.0 != 0 {
					outer.on_show(lparam.0)
				}
				else {
					outer.on_hide(lparam.0)
				}
			}
			WM_LBUTTONDOWN => outer.on_left_button_down(&MouseModifiers::from_wparam(wparam), &Position::from_lparam(lparam)),
			WM_LBUTTONUP => outer.on_left_button_up(&MouseModifiers::from_wparam(wparam), &Position::from_lparam(lparam)),
			WM_CLOSE => outer.on_close(),
			WM_DESTROY => outer.on_destroy(),
			_ => None
		}
	}

}

impl<W: WndBase> WndClass for WndClassImpl<W> {
	type WndType = W;

	fn register(&mut self, once: &'static OnceLock<Result<u16>>, class_name: &str, instance: Option<HINSTANCE>) -> Result<()> {
		self.register_with_init(once, class_name, instance, &|_| Ok(()))
	}

	fn register_with_init(&mut self,
		once: &'static OnceLock<Result<u16>>,
		class_name: &str,
		instance: Option<HINSTANCE>,
		init: &dyn Fn(&mut WNDCLASSW) -> Result<()>) -> Result<()> {

		self.instance = Some(instance.unwrap_or_else(|| unsafe { GetModuleHandleW(None) }
			.expect("GetModuleHandleW(None) should not fail").into()));

		once.get_or_init(|| {
			let mut class_name_w = [0u16; 258];
			StrConv::str_to_slice_w(class_name, &mut class_name_w, true);

			let mut class = WNDCLASSW {
				hCursor: unsafe { LoadCursorW(None, IDC_ARROW).ok().expect("Windows should provide the IDC_ARROW cursor") },
				hInstance: self.instance.unwrap(),
				lpszClassName: PCWSTR(class_name_w.as_ptr()),
				lpfnWndProc: Some(Self::wnd_proc),
				..Default::default()
			};

			(*init)(&mut class)?;

			match unsafe { RegisterClassW(&class) } {
				0  => Err(Error::from_win32()),
				atom => Ok(atom)
			}
		}).as_ref()
		.and_then(|a| {
			self.atom = Some(*a);
			Ok(())})
		.or_else(|e| {
			self.atom = None;
			Err(e.to_owned())
		})
	}

	fn create_window(&self, outer: Self::WndType, width: u32, height: u32, style: WINDOW_STYLE, ex_style: WINDOW_EX_STYLE, parent: Option<HWND>, menu: Option<HMENU>) -> Result<Rc<RefCell<Self::WndType>>> {
		self.atom.ok_or_else(|| Error::from_hresult(E_FAIL))
		.and_then(|atom| {
			super::adjust_window_size(width, height, style, ex_style)
			.or_else(|_| Err(Error::from_hresult(E_INVALIDARG)))
			.and_then(|(adjusted_width, adjusted_height)| {
				let ptr_fake: *const u16 = std::ptr::without_provenance(atom as usize);
				let class_name = PCWSTR::from_raw(ptr_fake);
				let title = HSTRING::from(outer.get_title().unwrap_or(""));
				let raw_ptr = Rc::into_raw(Rc::new(RefCell::new(outer)));

				unsafe {
					Rc::increment_strong_count(raw_ptr);
					let lpparam = Some(raw_ptr as *const std::ffi::c_void);

					CreateWindowExW(
						ex_style,
						class_name,
						&title,
						style,
						CW_USEDEFAULT,
						CW_USEDEFAULT,
						adjusted_width as i32,
						adjusted_height as i32,
						parent,
						menu,
						self.instance,
						lpparam,
					)
				}
				.and_then(|_| {
					let rc = unsafe { Rc::from_raw(raw_ptr).clone() };
					Ok(rc)
				})
			})
		})
	}

	unsafe extern "system" fn wnd_proc(handle: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
		unsafe {
			match message {
				WM_NCCREATE => {
					let create_struct = &*(lparam.0 as *const CREATESTRUCTW);
					let raw_ptr = create_struct.lpCreateParams as *const RefCell<Self::WndType>;
					SetWindowLongPtrW(handle, GWLP_USERDATA, raw_ptr as isize);
					Rc::increment_strong_count(raw_ptr);
					let rc = Rc::from_raw(raw_ptr);
					let mut outer = rc.borrow_mut();

					outer.set_handle(Some(handle));
					return LRESULT(1);
				}
				_ => {
					let raw_ptr = GetWindowLongPtrW(handle, GWLP_USERDATA) as *const RefCell<Self::WndType>;
					if raw_ptr != std::ptr::null_mut() {
						Rc::increment_strong_count(raw_ptr);
						let rc = Rc::from_raw(raw_ptr);
						if let Ok(Some(lresult)) =
							rc.try_borrow_mut()
							.and_then(|mut outer_mut|
								Ok(Self::on_message_mut(outer_mut.deref_mut(), message, wparam, lparam)))
							.or_else(|_| rc.try_borrow()
							.and_then(|outer|
								Ok(Self::on_message(outer.deref(), message, wparam, lparam)))) {
								if message == WM_DESTROY {
									Rc::decrement_strong_count(raw_ptr);
									SetWindowLongPtrW(handle, GWLP_USERDATA, 0);
								}
								return lresult;
						}
					}
					DefWindowProcW(handle, message, wparam, lparam)
				}
			}
		}
	}
}

