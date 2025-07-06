use std::{ffi::CStr, string::FromUtf16Error};


pub struct StrConv {
}

impl StrConv {
	pub fn utf16_copy(value: &str, target: &mut [u8]) -> usize {
		let mut pos = 0;
		for c in value.encode_utf16().take((target.len() / 2) - 1) {
			target[pos] = (c & 0xFFu16) as u8;
			target[pos + 1] = ((c >> 8) & 0xFFu16) as u8;
			pos += 2;
		}
		target[pos] = 0;
		target[pos + 1] = 0;
		return pos;
	}

	pub fn utf8_copy(value: &str, target: &mut [u8]) -> usize {
		match target.len() {
			0 => 0,
			1 => {
				target[0] = 0;
				0
			},
			len => {
				let mut bndry = usize::min(len - 1, value.len());
				while bndry > 0 && !value.is_char_boundary(bndry) {
					bndry -= 1;
				}
				if bndry > 0 {
					for (pos, c) in value.bytes().take(bndry).enumerate() {
						target[pos] = c;
					}
					target[bndry] = 0;
					bndry
				}
				else {
					0
				}
			}
		}
	}

	pub fn utf16_copy_w(value: &str, target: &mut [u16]) -> usize {
		let mut pos = 0;
		for c in value.encode_utf16().take(target.len() - 1) {
			target[pos] = c;
			pos += 1;
		}
		target[pos] = 0;
		return pos;
	}

	pub fn string_from(value: &[u8], is_utf16: bool) -> String {
		if is_utf16 {
			let mut pos = 0;
			let max = value.len();
			let mut conv: Vec<u16> = vec![0; max / 2];
			while pos < max - 1 {
				let codepoint = value[pos] as u16 | ((value[pos + 1] as u16) << 8);
				conv.push(codepoint);
				if codepoint == 0 {
					break;
				}
				pos += 2;
			}
			String::from_utf16(&conv).unwrap()
		} else {
			// TODO: Is THIS really required?!? Only to get all bytes before the zero and forward it?!?
			String::from_utf8(value.iter().map(|b| *b).take_while(|b| *b != 0u8).collect()).unwrap()
		}
	}

	pub fn w_from_str(value: &str, buffer: &mut [u16]) -> usize {
		let mut len = 0;

		for (idx, c) in value
			.encode_utf16()
			.take(buffer.len() - 1)
			.enumerate() {
			len += 1;
			buffer[idx] = c;
		}
		len
	}

	#[allow(dead_code)]
	pub fn string_from_w(value: &[u16]) -> String {
		let vec: Vec<u16> = value
			.iter()
			.map(|w| *w)
			.take_while(|w| *w != 0u16)
			.collect();
		String::from_utf16(&vec).unwrap()
	}

	pub fn as_fid_string(guid: &windows::core::GUID) -> String {
		// TODO: Check format (was: guid.to_string())
		format!("{:?}", guid)
	}

	pub fn c_str_cmp(a: *const u8, b: *const u8) -> isize {
		unsafe {
			let mut x = a;
			let mut y = b;

			loop {
				if *x == *y {
					if *x != 0u8 {
						x = x.add(1);
						y = y.add(1);
					}
					else {
						return 0;
					}
				}
				else if *x == 0u8 {
					return 1;
				}
				else if *y == 0u8 {
					return -1;
				}
			}
		}
	}

	pub fn str_to_c_str_vec(value: &str) -> Vec<u8> {
		let len = value.len();
		let mut buffer = vec![0u8; len + 1];
		buffer[0..len].copy_from_slice(value.as_bytes());
		buffer
	}

	pub fn c_str_to_vec(value: *const u8) -> Vec<u8> {
		let len = Self::c_str_len(value);
		let mut buffer = vec![0u8; len + 1];
		unsafe { value.copy_to(buffer.as_mut_ptr(), len) };
		buffer
	}

	pub fn c_str_copy(src: *const u8, dst: &mut [u8]) -> usize {
		let mut pos = src;
		let mut max = dst.len() - 1;

		for i in 0..max {
			dst[i] = unsafe {
				let c = *pos;
				pos = pos.add(1);
				c
			};
			if dst[i] == 0u8 {
				max = i;
				break;
			}
		}
		dst[max] = 0u8;

		max
	}

	pub fn c_str_len(value: *const u8) -> usize {
		let cstr = unsafe { CStr::from_ptr(value as *const std::ffi::c_char) };
		cstr.count_bytes()
	}

	pub fn c_str_to_string(value: *const u8) -> String {
		let cstr = unsafe { CStr::from_ptr(value as *const std::ffi::c_char) };
		cstr.to_string_lossy().into_owned()
	}

	pub fn w_str_copy(src: *const u16, dst: &mut [u16]) -> usize {
		let mut pos = src;
		let mut max = dst.len() - 1;

		for i in 0..max {
			dst[i] = unsafe {
				let c = *pos;
				pos = pos.add(1);
				c
			};
			if dst[i] == 0u16 {
				max = i;
				break;
			}
		}
		dst[max] = 0u16;

		max
	}

	pub fn w_str_len(value: *const u16) -> usize {
		let mut pos = value;
		let mut len = 0;

		loop {
			unsafe {
				if *pos == 0u16 {
					break;
				}
				pos = pos.add(1);
				len +=1;
			};
		}
		len
	}

	pub fn w_str_to_string(value: *const u16) -> Result<String, FromUtf16Error> {
		let len = StrConv::w_str_len(value);
		let mut buffer = vec![0u16; len];
		unsafe { value.copy_to(buffer.as_mut_ptr(), len) };
		String::from_utf16(buffer.as_slice())
	}
}


#[cfg(test)]
mod test {
	use super::StrConv;

	#[test]
	pub fn can_copy_utf16() {
		let value = "Thornton Wilder";
		let mut target = [0u8; 32];
		let byte_count = StrConv::utf16_copy(value, &mut target);

		assert_eq!(byte_count & 1, 0, "byte count should be an even number");

		let utf_16 : &[u16] = unsafe { std::slice::from_raw_parts((&target as *const u8) as *const u16, byte_count / 2) };
		let result = String::from_utf16_lossy(utf_16);

		assert_eq!(result, value, "copying as utf16 should be lossless");

		let mut odd_target = [0u8; 7];
		let byte_count = StrConv::utf16_copy("123", &mut odd_target);

		assert_eq!(byte_count, 4, "utf16_copy should use only even number of target bytes");
	}

	#[test]
	pub fn can_copy_utf8() {
		let mut target = [0u8; 16];

		for (value, expect) in [
			("Simple Text", "Simple Text"),
			// this is German for 'Beautiful shit'
			("Schöne Scheiße", "Schöne Scheiß"),
			// this is German for 'Hangs at the end'
			("Hängt am Ende drüber", "Hängt am Ende "),
			// this is Tulu for 'What do we have here?'
			("ನಮಕ್ ಮುಲ್ಪ ದಾದ ಉಂಡು?", "ನಮಕ್ "),
			("", "")] {
			let copied = StrConv::utf8_copy(value, &mut target);
			let result = String::from_utf8_lossy(&target[0..copied]);

			assert_eq!(result, expect);
		}
	}
}