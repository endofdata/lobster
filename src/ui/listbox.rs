use std::{cell::RefCell, rc::Rc};

use windows::{
	core::Result,
	Win32::{Foundation::{E_FAIL, HWND}, UI::WindowsAndMessaging::{WS_CHILD, WS_EX_CLIENTEDGE, WS_VISIBLE}}
};

use crate::ui::{Area, WndBase, WndClass, WndClassImpl};

pub struct ListBox {
	handle: Option<HWND>
}

impl ListBox {
	pub fn new(area: &Area, parent: HWND) -> Result<Rc<RefCell<Self>>> {
		WndClassImpl::<ListBox>::for_class("LISTBOX", None)
			.create_control(Self { handle: None }, area, WS_CHILD | WS_VISIBLE, WS_EX_CLIENTEDGE, parent)
	}
}

impl WndBase for ListBox {
	fn set_handle(&mut self, handle: Option<HWND>) {
		self.handle = handle;
	}

	fn get_handle(&self) -> Result<HWND> {
		self.handle.ok_or(windows::core::Error::from_hresult(E_FAIL))
	}
}