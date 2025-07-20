mod str_conv;

pub use str_conv::StrConv as StrConv;

#[rustfmt::skip]
use windows::{
	core::Result,
	Win32::System::Com::{CoInitializeEx, CoUninitialize, COINIT}
};

#[allow(dead_code)]
pub fn co_initialize_ex(co_init: Option<COINIT>) -> Result<()> {
	let co_init = co_init.unwrap_or_default();
	unsafe { CoInitializeEx(None, co_init) }.ok()
}

#[allow(dead_code)]
pub fn co_uninitialize() {
	unsafe { CoUninitialize() };
}
