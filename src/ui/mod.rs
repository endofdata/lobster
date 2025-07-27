mod wndclassimpl;
mod wndbase;
mod wndclass;
mod modifiers;
mod listbox;
mod geometry;

pub use self::wndclassimpl::WndClassImpl as WndClassImpl;
pub use self::wndbase::WndBase as WndBase;
pub use self::wndclass::WndClass as WndClass;
pub use self::modifiers::{MouseModifierFlags as MouseModifierFlags, MouseModifiers as MouseModifiers};
pub use self::geometry::{Vector2D as Vector2D, Area as Area};
pub use self::listbox::ListBox as ListBox;

#[rustfmt::skip]
use windows::{
	Win32::{
		Foundation::RECT,
		Graphics::Gdi::{GetStockObject, GET_STOCK_OBJECT_FLAGS, HBRUSH},
		UI::WindowsAndMessaging::{AdjustWindowRectEx, DispatchMessageW, GetMessageW, TranslateMessage,
			MSG, WINDOW_EX_STYLE, WINDOW_STYLE
		}
	},
	core::Result
};

#[allow(dead_code)]
pub fn adjust_window_size(size: &Vector2D, style: WINDOW_STYLE, ex_style: WINDOW_EX_STYLE) -> Result<Vector2D> {
	let mut rect : RECT = size.into() ;
	unsafe {
		AdjustWindowRectEx(&mut rect, style, false, ex_style)?;
	}
	Ok(rect.into())
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
