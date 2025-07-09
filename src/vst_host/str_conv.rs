use std::{ffi::CStr, string::FromUtf16Error};


pub struct StrConv {
}

impl StrConv {
	/// Write the utf-16 encoded `value` codepoints as little-endian byte pairs to `target`
	///
	/// If `zero_term` is `true` the `target` is always terminated with two `0u8`.
	pub fn str_to_bytes_w(value: &str, target: &mut [u8], zero_term: bool) -> usize {
		let mut pos = 0;
		let max = (target.len() - if zero_term { 1 } else {0}) / 2;
		if max > 0 {
			for c in value.encode_utf16().take(max) {
				target[pos] = (c & 0xFFu16) as u8;
				target[pos + 1] = ((c >> 8) & 0xFFu16) as u8;
				pos += 2;
			}
		}
		if zero_term {
			target[pos] = 0;
			target[pos + 1] = 0;
		}
		return pos;
	}

	/// Write `value` as utf-8 bytes to `target`
	///
	/// If `zero_term` is `true` target is always terminated with a single `0u8`.
	pub fn str_to_bytes(value: &str, target: &mut [u8], zero_term: bool) -> usize {
		let len = match target.len() - if zero_term { 1 } else  { 0 } {
			0 => {
				0
			},
			len => {
				let mut bndry = usize::min(len, value.len());
				while bndry > 0 && !value.is_char_boundary(bndry) {
					bndry -= 1;
				}
				if bndry > 0 {
					for (pos, c) in value.bytes().take(bndry).enumerate() {
						target[pos] = c;
					}
					bndry
				}
				else {
					0
				}
			}
		};
		if zero_term {
			target[len] = 0;
		}
		len
	}

	/// Creates a [String] from two-byte pairs, each read as little-endian utf-16 codepoint
	///
	/// Stops at the first byte pair of two zero-bytes or the end of `value`
	pub fn bytes_w_to_string(value: &[u8]) -> Option<String> {
		let max = value.chunks(2).count();
		let mut conv = Vec::with_capacity(max);

		for pair in value.chunks(2).take(max) {
			let codepoint = pair[0] as u16 | ((pair[1] as u16) << 8);
			if codepoint == 0 {
				break;
			}
			conv.push(codepoint);
		}
		String::from_utf16(&conv).ok()
	}

	/// Attempts to create a [String] from a portion of `value` up to the first `0u16`.
	///
	/// Returns `None` if `value` does not contain valid utf-16 encoded data
	#[allow(dead_code)]
	pub fn slice_w_to_string(value: &[u16]) -> Option<String> {
		String::from_utf16(value
			.iter()
			.take_while(|&&w| w != 0u16)
			.cloned()
			.collect::<Vec<u16>>()
			.as_slice())
		.ok()
	}

	/// Attempts to create a [String] from a portion of `value` up to the first `0u8`.
	///
	/// Returns `None` if `value` does not contain valid utf-8 encoded data
	pub fn slice_to_string(value: &[u8]) -> Option<String> {
		String::from_utf8(value
			.iter()
			.take_while(|&&b| b != 0u8)
			.cloned()
			.collect())
		.ok()
	}

	/// Write the utf-16 encoded `value` codepoints as little-endian byte pairs to `target`
	///
	/// If `zero_term` is `true` the `target` is always terminated with `0u16`.
	pub fn str_to_w_str(value: &str, target: *mut u16, max: usize, zero_term: bool) -> usize {
		if target != std::ptr::null_mut() {
			let utf_16 = value.encode_utf16();
			let count = usize::min(value.chars().count(), max - if zero_term { 1 } else { 0 });
			let mut pos = target;

			for c in utf_16.take(count) {
				unsafe {
					*pos = c;
					pos = pos.add(1);
				}
			}
			if zero_term {
				unsafe { *pos = 0u16 };
			}
			count
		}
		else {
			0
		}
	}

	/// Formats a `guid` as [String] in a format as expected by VST SDK
	pub fn as_fid_string(guid: &windows::core::GUID) -> String {
		// TODO: Check format (was: guid.to_string())
		format!("{:?}", guid)
	}

	/// Compares two raw, zero-terminated C-style strings
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

	/// Creates a vector that holds the `value`'s bytes terminated by a `0u8`
	pub fn str_to_c_str_vec(value: &str) -> Vec<u8> {
		let len = value.len();
		let mut buffer = vec![0u8; len + 1];
		buffer[0..len].copy_from_slice(value.as_bytes());
		buffer
	}

	/// Creates a vector from the bytes starting at `value` up to and optionally including the terminating `0u8`
	pub fn c_str_to_vec(value: *const u8, zero_term: bool) -> Vec<u8> {
		let len = Self::c_str_len(value);
		let mut buffer = vec![0u8; len + if zero_term { 1 } else { 0 }];
		unsafe { value.copy_to(buffer.as_mut_ptr(), len) };
		buffer
	}

	/// Copies the bytes starting at `value` up to and optionally including the terminating `0u8` to `target`
	pub fn c_str_to_slice(value: *const u8, target: &mut [u8], zero_term: bool) -> usize {
		let mut pos = value;
		let mut max = target.len() - if zero_term { 1 } else { 0 };

		for i in 0..max {
			target[i] = unsafe {
				let c = *pos;
				pos = pos.add(1);
				c
			};
			if target[i] == 0u8 {
				max = i;
				break;
			}
		}
		if zero_term {
			target[max] = 0u8;
		}
		max
	}

	/// Gets the number of bytes starting at `value` up to the first `0u8`
	pub fn c_str_len(value: *const u8) -> usize {
		let cstr = unsafe { CStr::from_ptr(value as *const std::ffi::c_char) };
		cstr.count_bytes()
	}

	/// Creates a [String] from the bytes starting at `value` up to the first `0u8`.
	pub fn c_str_to_string(value: *const u8) -> String {
		let cstr = unsafe { CStr::from_ptr(value as *const std::ffi::c_char) };
		cstr.to_string_lossy().into_owned()
	}

	pub fn str_to_slice_w(value: &str, target: &mut [u16], zero_term: bool) -> usize {
		let mut pos = 0;
		let max = target.len() - if zero_term { 1 } else { 0 };

		for c in value.encode_utf16().take(max) {
			target[pos] = c;
			pos += 1;
		}
		if zero_term {
			target[pos] = 0;
		}
		return pos;
	}

	pub fn w_str_to_slice_w(src: *const u16, dst: &mut [u16], zero_term: bool) -> usize {
		let mut pos = src;
		let mut max = dst.len() - if zero_term { 1 } else { 0};

		for i in 0..max {
			if dst[i] == 0u16 {
				max = i;
				break;
			}
			dst[i] = unsafe {
				let c = *pos;
				pos = pos.add(1);
				c
			};
		}
		if zero_term {
			dst[max] = 0u16;
		}
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
		let byte_count = StrConv::str_to_bytes(value, &mut target, false);

		assert_eq!(byte_count & 1, 0, "byte count should be an even number");

		let utf_16 : &[u16] = unsafe { std::slice::from_raw_parts((&target as *const u8) as *const u16, byte_count / 2) };
		let result = String::from_utf16_lossy(utf_16);

		assert_eq!(result, value, "copying as utf16 should be lossless");

		let mut odd_target = [0u8; 7];
		let byte_count = StrConv::str_to_bytes("123", &mut odd_target, false);

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
			let copied = StrConv::str_to_bytes(value, &mut target, false);
			let result = String::from_utf8_lossy(&target[0..copied]);

			assert_eq!(result, expect);
		}
	}
}