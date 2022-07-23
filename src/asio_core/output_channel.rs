use crate::asio_core::channel_iter_mut::ChannelIterMut;

pub struct OutputChannel<T> {
	pub name: String,
	pub ptr_a: *mut T,
	pub ptr_b: *mut T,
	len: usize
}

impl<T: Copy> OutputChannel<T> {
	pub fn new(name: &str, ptr_a: *mut T, ptr_b: *mut T, len: usize) -> OutputChannel<T> {
		OutputChannel {
			name: String::from(name),
			ptr_a: ptr_a,
			ptr_b: ptr_b,
			len: len,
		}
	}

	pub fn iter_mut(&mut self, double_buffer_index: i32) -> ChannelIterMut<T> {
		let write_second_half = double_buffer_index != 0;
		let ptr_current = match write_second_half {
			true => self.ptr_b,
			false => self.ptr_a
		};
		ChannelIterMut::new(ptr_current, self.len)
	}

	pub fn write(&mut self, double_buffer_index: i32, native_samples : &mut impl Iterator<Item = T>) {
		let avail = self.len;
		let target = &mut self.iter_mut(double_buffer_index);

		for native_sample in native_samples.take(avail) {
			match target.next() {
				Some(t) => {
					unsafe {
						*t = native_sample;
					}
				},
				None => break
			}
		}
	}
}

