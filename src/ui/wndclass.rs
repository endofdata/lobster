use std::sync::OnceLock;

#[rustfmt::skip]
use windows::{
	core::Result,
	Win32::{
		Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM},
		UI::WindowsAndMessaging::{HMENU, WINDOW_EX_STYLE, WINDOW_STYLE, WNDCLASSW}
	}
};

use super::WndBase;

pub trait WndClass {
	type WndType: WndBase;

	fn register(&mut self, once: &'static OnceLock<Result<u16>>, class_name: &str, instance: Option<HINSTANCE>) -> Result<()>;

	fn register_with_init(&mut self, once: &'static OnceLock<Result<u16>>, class_name: &str,
		instance: Option<HINSTANCE>, init: &dyn Fn(&mut WNDCLASSW) -> Result<()>) -> Result<()>;

	fn create_window(&self, outer: &mut Self::WndType, width: u32, height: u32, style: WINDOW_STYLE, ex_style: WINDOW_EX_STYLE,
		parent: Option<HWND>, menu: Option<HMENU>) -> Result<()>;

	unsafe extern "system" fn wnd_proc(handle: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT;
}
