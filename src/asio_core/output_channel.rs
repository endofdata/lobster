pub struct OutputChannel<T> {
	pub name: String,
	pub ptr_a: *mut T,
	pub ptr_b: *mut T,
	pub ptr_current: *mut T,
	len: usize,
	pos: usize
}

impl<T: Copy> OutputChannel<T> {
	pub fn new(name: &str, ptr_a: *mut T, ptr_b: *mut T, len: usize) -> OutputChannel<T> {
		OutputChannel {
			name: String::from(name),
			ptr_a: ptr_a,
			ptr_b: ptr_b,
			ptr_current: ptr_a,
			len: len,
			pos: 0
		}
	}

	pub fn select_buffer(&mut self, double_buffer_index: i32) {
		let write_second_half = double_buffer_index != 0;
		self.ptr_current = match write_second_half {
			true => self.ptr_b,
			false => self.ptr_a
		};
	}

	pub fn reset(&mut self) {
		self.pos = 0;
	}

	pub fn write(&mut self, samples : &mut impl Iterator<Item = T>) {
		let avail = self.len - self.pos;

		for native_sample in samples.take(avail) {
			unsafe {
				*self.ptr_current.offset(self.pos as isize) = native_sample;
			}
			self.pos += 1;
		}
	}
}

