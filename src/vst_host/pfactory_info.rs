use super::factory_flags::FactoryFlags;
use super::{utf8_copy, utf16_copy, string_from};

pub const URL_SIZE : usize = 256;
pub const EMAIL_SIZE : usize = 128;
pub const NAME_SIZE : usize = 64;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PFactoryInfo {
	vendor: [u8; NAME_SIZE],
	url: [u8; URL_SIZE],
	email: [u8; EMAIL_SIZE],
	flags: FactoryFlags
}

impl PFactoryInfo {
	pub fn new() -> PFactoryInfo {
		PFactoryInfo {
			vendor: [0; NAME_SIZE],
			url: [0; URL_SIZE],
			email: [0; EMAIL_SIZE],
			flags: FactoryFlags::NoFlags
		}
	}

	pub fn new_with(vendor: &str, url: &str, email: &str, flags: FactoryFlags) -> PFactoryInfo {
		let mut instance = PFactoryInfo::new();
		instance.init(vendor, url, email, flags);
		instance
	}

	pub fn init(&mut self, vendor: &str, url: &str, email: &str, flags: FactoryFlags) {
		if flags.has_flag(FactoryFlags::Unicode) {
			utf16_copy(vendor, &mut self.vendor);
			utf16_copy(url, &mut self.url);
			utf16_copy(email, &mut self.email);
		}
		else {
			utf8_copy(vendor, &mut self.vendor);
			utf8_copy(url, &mut self.url);
			utf8_copy(email, &mut self.email);
		}
		self.flags = flags;
	}

	pub fn get_vendor(&self) -> String {
		string_from(&self.vendor, false)
	}

	pub fn get_url(&self) -> String {
		string_from(&self.url, false)
	}

	pub fn get_email(&self) -> String {
		string_from(&self.email, false)
	}

	pub fn get_flags(&self) -> FactoryFlags {
		self.flags
	}

}