use crate::asio_core::channel_iter::ChannelIter;

pub struct InputChannel<T> {
	pub name: String,
	ptr_a: *const T,
	ptr_b: *const T,
	len: usize
}

impl<T: Copy> InputChannel<T> {
	pub fn new(name: &str, ptr_a: *const T, ptr_b: *const T, len: usize) -> InputChannel<T> {
		InputChannel {
			name: String::from(name),
			ptr_a: ptr_a,
			ptr_b: ptr_b,
			len: len
		}
	}

	pub fn iter(&mut self, double_buffer_index: i32) -> ChannelIter<T> {
		let read_second_half = double_buffer_index == 0;
		let ptr_current = match read_second_half {
			true => self.ptr_b,
			false => self.ptr_a
		};
		ChannelIter::<T>::new(ptr_current, self.len)
	}
}

