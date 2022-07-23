use std::marker::PhantomData;

pub struct ChannelIter<'a, T> {
	phantom: PhantomData<&'a T>,
	buffer: *const T,
	len: usize,
	pos: usize
}

impl<'a, T: Copy> ChannelIter<'a, T> {
	pub fn new(buffer: *const T, len: usize) -> ChannelIter<'a, T> {
		ChannelIter { 
			phantom: PhantomData,
			buffer: buffer, 
			len: len, 
			pos: 0 
		}
	}
}

impl<'a, T: Copy> Iterator for ChannelIter<'a, T> {
	type Item = T;

	fn next(&mut self) -> Option<T> {
		match self.len < self.pos {
			true => {
				let result;
				unsafe {
					result = Some(*self.buffer.offset(self.pos as isize));
				}
				self.pos += 1;
				result
			},
			false => None
		}
	}
}
