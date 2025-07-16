use windows::Win32::System::Threading::GetCurrentThreadId;

#[derive(Copy, Clone)]
pub struct ThreadCheck(u32);

impl ThreadCheck {
	pub fn for_current_thread() -> Self {
		Self(Self::get_current_thread_id())
	}

	pub fn is_expected(&self) -> bool {
		Self::get_current_thread_id() == self.0
	}

	pub fn with_expected<U, T: FnOnce() -> U>(&self, f: T) -> Option<U> {
		if self.is_expected() {
			Some(f())
		}
		else {
			None
		}
	}

	fn get_current_thread_id() -> u32 {
		unsafe { GetCurrentThreadId() }
	}
}