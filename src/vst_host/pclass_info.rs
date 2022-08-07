use super::{class_cardinality::ClassCardinality, utf8_copy, utf16_copy_w, empty_guid, string_from, string_from_w};

pub const CATEGORY_SIZE : usize = 32;
pub const NAME_SIZE : usize = 64;
pub const VENDOR_SIZE : usize = 64;
pub const VERSION_SIZE : usize = 64;
pub const SUBCATEGORY_SIZE : usize = 128;

#[repr(C)]
pub struct PClassInfo {
	cid: com::IID,
	cardinality: ClassCardinality,
	category: [u8; CATEGORY_SIZE],
	name: [u8; NAME_SIZE]
}

impl PClassInfo {
	pub fn new() -> PClassInfo {
		PClassInfo { 
			cid: empty_guid(),
			cardinality: ClassCardinality::NoValue, 
			category: [0; CATEGORY_SIZE], 
			name: [0; NAME_SIZE]
		}
	}

	pub fn new_with(cid: &com::IID, cardinality: ClassCardinality, category: &str, name: &str) -> PClassInfo {
		let mut instance = PClassInfo::new();
		instance.init(cid, cardinality, category, name);
		instance
	}

	pub fn init(&mut self, cid: &com::IID, cardinality: ClassCardinality, category: &str, name: &str) {
		self.cid = *cid;
		self.cardinality = cardinality;
		utf8_copy(category, &mut self.category);
		utf8_copy(name, &mut self.name);
	}
}

#[repr(C)]
pub struct PClassInfo2 {
	cid: com::IID,
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
			cid: empty_guid(),
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

	pub fn new_with(cid: &com::IID, cardinality: ClassCardinality, category: &str, name: &str,
		class_flags: u32, sub_categories: &str, vendor: &str, version: &str, sdk_version: &str) -> PClassInfo2 {
		let mut instance = PClassInfo2::new();
		instance.init(cid, cardinality, category, name, class_flags, sub_categories, vendor, version, sdk_version);
		instance
	}

	pub fn init(&mut self, cid: &com::IID, cardinality: ClassCardinality, category: &str, name: &str,
		class_flags: u32, sub_categories: &str, vendor: &str, version: &str, sdk_version: &str) {
		self.cid = *cid;
		self.cardinality = cardinality;
		utf8_copy(category, &mut self.category);
		utf8_copy(name, &mut self.name);
		self.class_flags = class_flags;
		utf8_copy(sub_categories, &mut self.sub_categories);
		utf8_copy(vendor, &mut self.vendor);
		utf8_copy(version, &mut self.version);
		utf8_copy(sdk_version, &mut self.sdk_version);
	}
}


#[repr(C)]
pub struct PClassInfoW {
	cid: com::IID,
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
			cid: empty_guid(),
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

	pub fn new_with(cid: &com::IID, cardinality: ClassCardinality, category: &str, name: &str,
		class_flags: u32, sub_categories: &str, vendor: &str, version: &str, sdk_version: &str) -> PClassInfoW {
		let mut instance = PClassInfoW::new();
		instance.init(cid, cardinality, category, name, class_flags, sub_categories, vendor, version, sdk_version);
		instance
	}

	pub fn init(&mut self, cid: &com::IID, cardinality: ClassCardinality, category: &str, name: &str,
		class_flags: u32, sub_categories: &str, vendor: &str, version: &str, sdk_version: &str) {
		self.cid = *cid;
		self.cardinality = cardinality;
		utf8_copy(category, &mut self.category);
		utf16_copy_w(name, &mut self.name);
		self.class_flags = class_flags;
		utf8_copy(sub_categories, &mut self.sub_categories);
		utf16_copy_w(vendor, &mut self.vendor);
		utf16_copy_w(version, &mut self.version);
		utf16_copy_w(sdk_version, &mut self.sdk_version);
	}
}


#[allow(dead_code)]
#[derive(Debug)]
pub struct ClassInfo {
	cid: com::IID,
	cardinality: ClassCardinality,
	class_flags: u32,
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
			category: string_from(&src.category, false), 
			name: string_from(&src.name, false), 
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
			category: string_from(&src.category, false), 
			name: string_from(&src.name, false), 
			sub_categories: string_from(&src.sub_categories, false), 
			vendor: string_from(&src.vendor, false), 
			version: string_from(&src.version, false), 
			sdk_version: string_from(&src.sdk_version, false)
		}
	}

	pub fn from_class_info_w(src: &PClassInfoW) -> ClassInfo {
		ClassInfo { 
			cid: src.cid, 
			cardinality: src.cardinality, 
			class_flags: src.class_flags, 
			category: string_from(&src.category, false), 
			name: string_from_w(&src.name), 
			sub_categories: string_from(&src.sub_categories, false), 
			vendor: string_from_w(&src.vendor), 
			version: string_from_w(&src.version), 
			sdk_version: string_from_w(&src.sdk_version)
		}
	}
}