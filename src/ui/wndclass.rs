use std::{cell::RefCell, rc::Rc, sync::OnceLock};

#[rustfmt::skip]
use windows::{
	core::Result,
	Win32::{
		Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM},
		UI::WindowsAndMessaging::{HMENU, WINDOW_EX_STYLE, WINDOW_STYLE, WNDCLASSW}
	}
};

use crate::ui::Area;

use super::WndBase;

pub trait WndClass {
	type WndType: WndBase;

	fn register(&mut self, once: &'static OnceLock<Result<u16>>, class_name: &str, instance: Option<HINSTANCE>) -> Result<()>;

	fn register_with_init(&mut self, once: &'static OnceLock<Result<u16>>, class_name: &str,
		instance: Option<HINSTANCE>, init: &dyn Fn(&mut WNDCLASSW) -> Result<()>) -> Result<()>;

	fn create_window(&self, outer: Self::WndType, area: &Area, style: WINDOW_STYLE, ex_style: WINDOW_EX_STYLE,
		parent: Option<HWND>, menu: Option<HMENU>) -> Result<Rc<RefCell<Self::WndType>>>;

	fn create_control(&self, outer: Self::WndType, area: &Area, style: WINDOW_STYLE, ex_style: WINDOW_EX_STYLE,
		parent: HWND) -> Result<Rc<RefCell<Self::WndType>>>;

	unsafe extern "system" fn wnd_proc(handle: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT;
}
