use windows::{
	core::{implement, Interface, HRESULT, Result},
	Win32::Foundation::{E_FAIL, E_INVALIDARG, S_OK}
};

use crate::vst_host::{IPlugFrame, IPlugFrame_Impl, IPlugView, ViewRect};

pub trait Resizable {
	fn resize_view(&self, view: &IPlugView, new_size: &ViewRect) -> Result<()>;
}

#[implement(IPlugFrame)]
pub struct PlugFrame<'a>(&'a dyn Resizable);

impl<'a> PlugFrame<'a> {
	pub fn new<R: Resizable>(plug_wnd: &'a R) -> Self {
		Self(plug_wnd)
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
