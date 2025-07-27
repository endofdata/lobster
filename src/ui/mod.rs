mod wndclassimpl;
mod wndbase;
mod wndclass;
mod modifiers;

pub use self::wndclassimpl::WndClassImpl as WndClassImpl;
pub use self::wndbase::WndBase as WndBase;
pub use self::wndclass::WndClass as WndClass;
pub use self::modifiers::{MouseModifierFlags as MouseModifierFlags, MouseModifiers as MouseModifiers, Position as Position};

#[rustfmt::skip]
use windows::{
	Win32::{
		Foundation::{E_INVALIDARG, RECT},
		Graphics::Gdi::{GetStockObject, GET_STOCK_OBJECT_FLAGS, HBRUSH},
		UI::WindowsAndMessaging::{AdjustWindowRectEx, DispatchMessageW, GetMessageW, TranslateMessage,
			MSG, WINDOW_EX_STYLE, WINDOW_STYLE
		}
	},
	core::{Result, Error}
};

#[allow(dead_code)]
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

#[allow(dead_code)]
pub fn run_message_loop() {
	let mut msg = MSG::default();

	unsafe {
		while GetMessageW(&mut msg, None, 0, 0).into() {
			_ = TranslateMessage(&msg);
			DispatchMessageW(&msg);
		}
	}
}

#[allow(dead_code)]
pub fn get_stock_brush(flags: GET_STOCK_OBJECT_FLAGS) -> HBRUSH {
	unsafe { HBRUSH(GetStockObject(flags).0) }
}
