use super::{class_cardinality::ClassCardinality, utf8_copy, utf16_copy_w, empty_guid};

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
