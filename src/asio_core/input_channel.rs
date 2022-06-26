pub struct InputChannel<T> {
	pub name: String,
	ptr_a: *const T,
	ptr_b: *const T,
	ptr_current: *const T,
	len: usize,
	pos: usize
}

impl<T: Copy> InputChannel<T> {
	pub fn new(name: &str, ptr_a: *const T, ptr_b: *const T, len: usize) -> InputChannel<T> {
		InputChannel {
			name: String::from(name),
			ptr_a: ptr_a,
			ptr_b: ptr_b,
			ptr_current: ptr_a,
			len: len,
			pos: 0
		}
	}

	pub fn select_buffer(&mut self, double_buffer_index: i32) {
		let read_second_half = double_buffer_index == 0;
		self.ptr_current = match read_second_half {
			true => self.ptr_b,
			false => self.ptr_a
		};
	}

	pub fn reset(&mut self) {
		self.pos = 0;
	}
}

impl<T: Copy> Iterator for InputChannel<T> {
	type Item = T;

	fn next(&mut self) -> Option<T> {
		match self.len < self.pos {
			true => {
				let result;
				unsafe {
					result = Some(*self.ptr_current.offset(self.pos as isize));
				}
				self.pos += 1;
				result
			},
			false => None
		}
	}
}
