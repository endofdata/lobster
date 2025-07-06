use super::{class_cardinality::ClassCardinality, str_conv::StrConv };
use windows::core::GUID;

pub const CATEGORY_SIZE : usize = 32;
pub const NAME_SIZE : usize = 64;
pub const VENDOR_SIZE : usize = 64;
pub const VERSION_SIZE : usize = 64;
pub const SUBCATEGORY_SIZE : usize = 128;

#[repr(C)]
pub struct PClassInfo {
	cid: GUID,
	cardinality: ClassCardinality,
	category: [u8; CATEGORY_SIZE],
	name: [u8; NAME_SIZE]
}

impl PClassInfo {
	pub fn new() -> PClassInfo {
		PClassInfo {
			cid: GUID::zeroed(),
			cardinality: ClassCardinality::NoValue,
			category: [0; CATEGORY_SIZE],
			name: [0; NAME_SIZE]
		}
	}

	pub fn new_with(cid: &GUID, cardinality: ClassCardinality, category: &str, name: &str) -> PClassInfo {
		let mut instance = PClassInfo::new();
		instance.init(cid, cardinality, category, name);
		instance
	}

	pub fn init(&mut self, cid: &GUID, cardinality: ClassCardinality, category: &str, name: &str) {
		self.cid = *cid;
		self.cardinality = cardinality;
		StrConv::utf8_copy(category, &mut self.category);
		StrConv::utf8_copy(name, &mut self.name);
	}
}

#[repr(C)]
pub struct PClassInfo2 {
	cid: GUID,
	cardinality: ClassCardinality,
	category: [u8; CATEGORY_SIZE],
	name: [u8; NAME_SIZE],
	class_flags: u32,
	sub_categories: [u8; SUBCATEGORY_SIZE],
	vendor: [u8; VENDOR_SIZE],
	version: [u8; VERSION_SIZE],
	sdk_version: [u8; VERSION_SIZE]
}

impl PClassInfo2 {
	pub fn new() -> PClassInfo2 {
		PClassInfo2 {
			cid: GUID::zeroed(),
			cardinality: ClassCardinality::NoValue,
			category: [0; CATEGORY_SIZE],
			name: [0; NAME_SIZE],
			class_flags: 0,
			sub_categories: [0; SUBCATEGORY_SIZE],
			vendor: [0; VENDOR_SIZE],
			version: [0; VERSION_SIZE],
			sdk_version: [0; VERSION_SIZE]
		}
	}

	pub fn new_with(cid: &GUID, cardinality: ClassCardinality, category: &str, name: &str,
		class_flags: u32, sub_categories: &str, vendor: &str, version: &str, sdk_version: &str) -> PClassInfo2 {
		let mut instance = PClassInfo2::new();
		instance.init(cid, cardinality, category, name, class_flags, sub_categories, vendor, version, sdk_version);
		instance
	}

	pub fn init(&mut self, cid: &GUID, cardinality: ClassCardinality, category: &str, name: &str,
		class_flags: u32, sub_categories: &str, vendor: &str, version: &str, sdk_version: &str) {
		self.cid = *cid;
		self.cardinality = cardinality;
		StrConv::utf8_copy(category, &mut self.category);
		StrConv::utf8_copy(name, &mut self.name);
		self.class_flags = class_flags;
		StrConv::utf8_copy(sub_categories, &mut self.sub_categories);
		StrConv::utf8_copy(vendor, &mut self.vendor);
		StrConv::utf8_copy(version, &mut self.version);
		StrConv::utf8_copy(sdk_version, &mut self.sdk_version);
	}
}


#[repr(C)]
pub struct PClassInfoW {
	cid: GUID,
	cardinality: ClassCardinality,
	category: [u8; CATEGORY_SIZE],
	name: [u16; NAME_SIZE],
	class_flags: u32,
	sub_categories: [u8; SUBCATEGORY_SIZE],
	vendor: [u16; VENDOR_SIZE],
	version: [u16; VERSION_SIZE],
	sdk_version: [u16; VERSION_SIZE]
}

impl PClassInfoW {
	pub fn new() -> PClassInfoW {
		PClassInfoW {
			cid: GUID::zeroed(),
			cardinality: ClassCardinality::NoValue,
			category: [0; CATEGORY_SIZE],
			name: [0; NAME_SIZE],
			class_flags: 0,
			sub_categories: [0; SUBCATEGORY_SIZE],
			vendor: [0; VENDOR_SIZE],
			version: [0; VERSION_SIZE],
			sdk_version: [0; VERSION_SIZE]
		}
	}

	pub fn new_with(cid: &GUID, cardinality: ClassCardinality, category: &str, name: &str,
		class_flags: u32, sub_categories: &str, vendor: &str, version: &str, sdk_version: &str) -> PClassInfoW {
		let mut instance = PClassInfoW::new();
		instance.init(cid, cardinality, category, name, class_flags, sub_categories, vendor, version, sdk_version);
		instance
	}

	pub fn init(&mut self, cid: &GUID, cardinality: ClassCardinality, category: &str, name: &str,
		class_flags: u32, sub_categories: &str, vendor: &str, version: &str, sdk_version: &str) {
		self.cid = *cid;
		self.cardinality = cardinality;
		StrConv::utf8_copy(category, &mut self.category);
		StrConv::utf16_copy_w(name, &mut self.name);
		self.class_flags = class_flags;
		StrConv::utf8_copy(sub_categories, &mut self.sub_categories);
		StrConv::utf16_copy_w(vendor, &mut self.vendor);
		StrConv::utf16_copy_w(version, &mut self.version);
		StrConv::utf16_copy_w(sdk_version, &mut self.sdk_version);
	}
}


#[allow(dead_code)]
#[derive(Debug)]
pub struct ClassInfo {
	pub cid: GUID,
	pub cardinality: ClassCardinality,
	pub class_flags: u32,
	pub category: String,
	pub name: String,
	pub sub_categories: String,
	pub vendor: String,
	pub version: String,
	pub sdk_version: String
}

impl ClassInfo {
	pub fn from_class_info(src: &PClassInfo) -> ClassInfo {
		ClassInfo {
			cid: src.cid,
			cardinality: src.cardinality,
			class_flags: 0,
			category: StrConv::string_from(&src.category, false),
			name: StrConv::string_from(&src.name, false),
			sub_categories: String::new(),
			vendor: String::new(),
			version: String::new(),
			sdk_version: String::new()
		}
	}

	pub fn from_class_info_2(src: &PClassInfo2) -> ClassInfo {
		ClassInfo {
			cid: src.cid,
			cardinality: src.cardinality,
			class_flags: src.class_flags,
			category: StrConv::string_from(&src.category, false),
			name: StrConv::string_from(&src.name, false),
			sub_categories: StrConv::string_from(&src.sub_categories, false),
			vendor: StrConv::string_from(&src.vendor, false),
			version: StrConv::string_from(&src.version, false),
			sdk_version: StrConv::string_from(&src.sdk_version, false)
		}
	}

	pub fn from_class_info_w(src: &PClassInfoW) -> ClassInfo {
		ClassInfo {
			cid: src.cid,
			cardinality: src.cardinality,
			class_flags: src.class_flags,
			category: StrConv::string_from(&src.category, false),
			name: StrConv::string_from_w(&src.name),
			sub_categories: StrConv::string_from(&src.sub_categories, false),
			vendor: StrConv::string_from_w(&src.vendor),
			version: StrConv::string_from_w(&src.version),
			sdk_version: StrConv::string_from_w(&src.sdk_version)
		}
	}
}
