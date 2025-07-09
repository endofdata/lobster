use super::factory_flags::FactoryFlags;
use super::str_conv::StrConv;

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
			StrConv::str_to_bytes(vendor, &mut self.vendor, true);
			StrConv::str_to_bytes(url, &mut self.url, true);
			StrConv::str_to_bytes(email, &mut self.email, true);
		}
		else {
			StrConv::str_to_bytes(vendor, &mut self.vendor, true);
			StrConv::str_to_bytes(url, &mut self.url, true);
			StrConv::str_to_bytes(email, &mut self.email, true);
		}
		self.flags = flags;
	}

	pub fn get_vendor(&self) -> Option<String> {
		StrConv::slice_to_string(&self.vendor)
	}

	pub fn get_url(&self) -> Option<String> {
		StrConv::slice_to_string(&self.url)
	}

	pub fn get_email(&self) -> Option<String> {
		StrConv::slice_to_string(&self.email)
	}

	pub fn get_flags(&self) -> FactoryFlags {
		self.flags
	}

}