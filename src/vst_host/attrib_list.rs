use std::cell::RefCell;
use std::collections::HashMap;

use windows::{
	Win32::Foundation::{E_FAIL, E_INVALIDARG, S_OK},
	core::{implement, HRESULT}
};

use super::{
	AttrID, IAttributeList, IAttributeList_Impl,

};

use crate::os::StrConv;

const MAX_ATTRKEY_LEN: usize = 64;

#[derive(Eq, Hash, PartialEq)]
pub struct AttrKey([u8;MAX_ATTRKEY_LEN]);

impl From<AttrID> for AttrKey {
	fn from(value: AttrID) -> Self {
		let mut buffer = [0u8; MAX_ATTRKEY_LEN];
		StrConv::c_str_to_slice(value, &mut buffer, true);
		AttrKey(buffer)
	}
}

#[derive(Clone)]
pub enum AttrValue {
	Integer(i64),
	Float(f64),
	String(String),
	Binary(Vec<u8>),
}

impl AttrValue {
	pub fn from_w_str(cstr: *const u16) -> Option<AttrValue> {
		match StrConv::w_str_to_string(cstr) {
			Ok(text) => Some(AttrValue::String(text)),
			Err(_) => None
		}
	}

	pub fn from_binary(data: *const std::ffi::c_void, size: usize) -> Option<AttrValue> {
		let mut value = Vec::<u8>::with_capacity(size);
		unsafe { value.as_mut_ptr().copy_from_nonoverlapping(data as *const u8, size) };

		Some(AttrValue::Binary(value))
	}
}


#[implement(IAttributeList)]
pub struct AttributeList {
	values: RefCell<HashMap<AttrKey, AttrValue>>
}

impl AttributeList {
	pub fn new() -> Self {
		println!("New attribute list");
		Self {
			values: RefCell::new(HashMap::<AttrKey, AttrValue>::new())
		}
	}

	pub fn set_attribute(&self, id: &AttrID, value: AttrValue) -> HRESULT {
		let _opt_old_value = self.values.borrow_mut().insert((*id).into(), value);
		S_OK
	}

	pub fn get_attribute(&self, id: &AttrID) -> Option<AttrValue> {
		let key : AttrKey = (*id).into();
		if let Some(x) = self.values.borrow().get(&key) {
			Some((*x).to_owned())
		}
		else {
			None
		}
	}
}

impl Drop for AttributeList {
	fn drop(&mut self) {
		println!("Dropping attribute list");
	}
}

impl IAttributeList_Impl for AttributeList_Impl {
	unsafe fn setInt(&self, id: AttrID, value: i64) -> HRESULT {
		self.set_attribute(&id, AttrValue::Integer(value))
	}

	unsafe fn getInt(&self, id: AttrID, value: *mut i64) -> HRESULT {
		match self.get_attribute(&id) {
			Some(AttrValue::Integer(x)) => {
				unsafe { *value = x };
				S_OK
			},
			_ => E_FAIL
		}
	}

	unsafe fn setFloat(&self, id: AttrID, value: f64) -> HRESULT {
		self.set_attribute(&id, AttrValue::Float(value))
	}

	unsafe fn getFloat(&self, id: AttrID, value: *mut f64) -> HRESULT {
		match self.get_attribute(&id) {
			Some(AttrValue::Float(x)) => {
				unsafe { *value = x };
				S_OK
			},
			_ => E_FAIL
		}
	}

	unsafe fn setString(&self, id: AttrID, w_str: *const u16) -> HRESULT {
		match AttrValue::from_w_str(w_str) {
			Some(value) => self.set_attribute(&id, value),
			None => E_INVALIDARG
		}
	}

	unsafe fn getString(&self, id: AttrID, w_str: *mut u16, sizeInBytes: u32) -> HRESULT {
		match self.get_attribute(&id) {
			Some(AttrValue::String(s)) => {
				let mut pos = w_str;
				for c in s.encode_utf16()
					.into_iter()
					.take((sizeInBytes as usize / std::mem::size_of::<u16>()) - 1) {
					unsafe { *pos = c; pos = pos.add(1); }
				}
				unsafe { *pos = 0u16 };
				S_OK
			},
			_ => E_FAIL
		}
	}

	unsafe fn setBinary(&self, id: AttrID, data: *const std::ffi::c_void, sizeInBytes: u32) -> HRESULT {
		match AttrValue::from_binary(data, sizeInBytes as usize) {
			Some(value) => self.set_attribute(&id.into(), value),
			None => E_FAIL
		}
	}

	unsafe fn getBinary(&self, id: AttrID, data: *mut std::ffi::c_void, sizeInBytes: &u32) -> HRESULT {
		match self.get_attribute(&id) {
			Some(AttrValue::Binary(vec)) => {
				let available = vec.len();
				if available <= *sizeInBytes as usize {
					unsafe { vec.as_ptr().copy_to_nonoverlapping(data as *mut u8, available) };
					S_OK
				}
				else {
					E_INVALIDARG
				}
			},
			_ => E_FAIL
		}
	}
}
