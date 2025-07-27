use windows::Win32::Foundation::WPARAM;


/// The CTRL key is down.
pub const MK_CONTROL : u16 = 0x0008;
/// The left mouse button is down.
pub const MK_LBUTTON : u16 = 0x0001;
/// The middle mouse button is down.
pub const MK_MBUTTON : u16 = 0x0010;
/// The right mouse button is down.
pub const MK_RBUTTON : u16 = 0x0002;
/// The SHIFT key is down.
pub const MK_SHIFT : u16 = 0x0004;
/// The XBUTTON1 is down.
pub const MK_XBUTTON1 : u16 = 0x0020;
/// The XBUTTON2 is down.
pub const MK_XBUTTON2 : u16 = 0x0040;

pub struct MouseModifiers(u16);

#[repr(u16)]
#[derive(Copy,Clone,Debug)]
#[allow(dead_code)]
pub enum MouseModifierFlags {
	None,
	Control = MK_CONTROL,
	LeftButton = MK_LBUTTON,
	MiddleButton = MK_MBUTTON,
	RightButton = MK_RBUTTON,
	Shift = MK_SHIFT,
	XButton1 = MK_XBUTTON1,
	XButton2 = MK_XBUTTON2,
}

impl MouseModifiers {
	#[allow(dead_code)]
	pub fn has(&self, flags: MouseModifierFlags) -> bool {
		((flags as u16) & self.0) != 0
	}
}

impl From<WPARAM> for MouseModifiers {
	fn from(value: WPARAM) -> Self {
		MouseModifiers(value.0 as u16)
	}
}