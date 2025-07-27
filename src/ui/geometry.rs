use windows::Win32::Foundation::{LPARAM, RECT};


#[derive(Copy, Clone, Debug, Default)]
pub struct Vector2D {
	x: i32,
	y: i32
}

impl Vector2D {
	pub fn new(x: i32, y: i32) -> Self {
		Self { x, y }
	}

	#[allow(dead_code)]
	pub fn get_x(&self) -> i32 {
		self.x
	}

	#[allow(dead_code)]
	pub fn get_y(&self) -> i32 {
		self.y
	}
}

impl From<LPARAM> for Vector2D {
	fn from(value: LPARAM) -> Self {
		Self { x: (value.0 & 0xffff) as i32, y: ((value.0 >> 16) & 0xffff) as i32 }
	}
}

impl From<&Vector2D> for RECT {
	fn from(value: &Vector2D) -> Self {
		Self { left: 0, top: 0, right: value.x, bottom: value.y }
	}
}

impl From<&Vector2D> for Area {
	fn from(value: &Vector2D) -> Self {
		Self::new(0, 0, value.x, value.y)
	}
}

impl From<&RECT> for Vector2D {
	fn from(value: &RECT) -> Self {
		Self { x: value.right - value.left, y: value.bottom - value.top }
	}
}

impl From<RECT> for Vector2D {
	fn from(value: RECT) -> Self {
		Self { x: value.right - value.left, y: value.bottom - value.top }
	}
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Area {
	offset: Vector2D,
	size: Vector2D
}

impl Area {
	pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
		Self { offset: Vector2D::new(x, y), size: Vector2D::new(width, height) }
	}

	pub fn get_offset(&self) -> &Vector2D {
		&self.offset
	}

	pub fn get_size(&self) -> &Vector2D {
		&self.size
	}
}

impl From<&RECT> for Area {
	fn from(value: &RECT) -> Self {
		Self::new(
			value.left,
			value.top,
			value.right - value.left,
			value.bottom - value.top)
	}
}

impl From<&Area> for RECT {
	fn from(value: &Area) -> Self {
		Self {
			left: value.offset.x,
			top: value.offset.y,
			right: value.offset.x + value.size.x,
			bottom: value.offset.y + value.size.y
		}
	}
}