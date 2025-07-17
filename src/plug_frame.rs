use windows::
	core::{implement, HRESULT}
;
use windows::Win32::Foundation::{E_FAIL, E_INVALIDARG, S_OK};
use windows_core::Interface;

use crate::vst_host::{IPlugFrame, IPlugFrame_Impl, IPlugView, ViewRect};
use crate::appwnd::AppWindow;

#[implement(IPlugFrame)]
pub struct PlugFrame<'a>(&'a AppWindow);

impl <'a> PlugFrame<'a> {
	pub fn new(app_wnd: &'a AppWindow) -> Self {
		Self(app_wnd)
	}
}

impl<'a> IPlugFrame_Impl for PlugFrame_Impl<'a> {
	#[allow(non_snake_case)]
	unsafe fn resizeView(&self, view: *const IPlugView, newSize: *const ViewRect) -> HRESULT {
		if view == std::ptr::null() || newSize == std::ptr::null() {
			E_INVALIDARG
		}
		else {
			let view_void = view as *mut std::ffi::c_void;
			let plug_view : &IPlugView = unsafe { IPlugView::from_raw_borrowed(&view_void) }
				.expect("Failed to borrow IPlugView from *const IPlugView");

			let new_size = unsafe { newSize.as_ref() }
				.expect("Failed to borrow ViewRect from *const ViewRect");

			match self.0.resize_view(plug_view, new_size) {
				Ok(()) => S_OK,
				_ => E_FAIL
			}
		}
	}
}
