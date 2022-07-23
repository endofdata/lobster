use std::marker::PhantomData;

pub struct ChannelIterMut<'a, T> {
	phantom: PhantomData<&'a T>,
	buffer: *mut T,
	len: usize,
	pos: usize
}

impl<'a, T: Copy> ChannelIterMut<'a, T> {
	pub fn new(buffer: *mut T, len: usize) -> ChannelIterMut<'a, T> {
		ChannelIterMut { 
			phantom: PhantomData, 
			buffer: buffer, 
			len: len, 
			pos: 0 
		}
	}
}

impl<'a, T: Copy> Iterator for ChannelIterMut<'a, T> {
	type Item = *mut T;

	fn next(&mut self) -> Option<Self::Item> {
		match self.len < self.pos {
			true => {
				let result;
				unsafe {
					result = Some(self.buffer.offset(self.pos as isize));
				}
				self.pos += 1;
				result
			},
			false => None
		}
	}
}