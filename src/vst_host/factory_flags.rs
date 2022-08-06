use std::ops::BitAnd;

#[derive(PartialEq, Copy, Clone)]
pub enum FactoryFlags {
	NoFlags = 0,
	ClassesDiscardable = 1,
	LicenseCheck = 1 << 1,
	ComponentNonDiscardable = 1 << 3,
	Unicode = 1 << 4
}

impl FactoryFlags {
	pub fn has_flag(&self, value: FactoryFlags) -> bool {
		self.bitand(value) == value
	}
}

/// How to make a [Flags] enum
impl BitAnd for FactoryFlags {
	type Output = FactoryFlags;

	fn bitand(self, rhs: Self) -> Self::Output {
		let me = self as i32;
		let other = rhs as i32;
		let result = me & other;

		unsafe {
			let ptr = std::ptr::addr_of!(result);
			*ptr.cast::<FactoryFlags>()
		}
	}
}