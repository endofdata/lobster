use std::cell::RefCell;

use windows::core::{implement, ComObject};

use super::{attrib_list::AttributeList, FIDString, IAttributeList, IMessage, IMessage_Impl};
use crate::os::StrConv;


#[implement(IMessage)]
pub struct Message {
	id: RefCell<Option<Vec<u8>>>,
	attribs: IAttributeList
}

impl Message {
	pub fn new() -> Self {
		println!("New message");
		Self {
			id: RefCell::new(None),
			attribs: ComObject::new(AttributeList::new()).cast()
				.expect("AttributeList should implement IAttributeList")
		}
	}

	pub fn with_id(id: &str) -> Self {
		println!("New message with id '{}'", id);
		Self {
			id: RefCell::new(Some(StrConv::str_to_c_str_vec(id))),
			attribs: ComObject::new(AttributeList::new()).cast()
				.expect("AttributeList should implement IAttributeList")
		}
	}
}

impl Drop for Message {
	fn drop(&mut self) {
		println!("Dropping message");
	}
}

impl IMessage_Impl for Message_Impl {
	unsafe fn getMessageID(&self) -> FIDString {
		if let Some(vec) = self.id.borrow().as_ref() {
			vec.as_ptr()
		}
		else {
			std::ptr::null()
		}
	}

	unsafe fn setMessageID(&self, id: FIDString) {
		*self.id.borrow_mut() =
			if id == std::ptr::null() {
				None
			}
			else {
				Some(StrConv::c_str_to_vec(id, true))
			}
	}

	unsafe fn getAttributes(&self) ->  *const IAttributeList {
		unsafe {std::mem::transmute_copy(&self.attribs) }
	}
}

#[cfg(test)]
mod test {
	use windows_core::{w, Interface, ComObject};

	use super::
	{
		Message,
		super::{AttrID, IAttributeList, IMessage}
	};
	use crate::os::StrConv;

	const TEST_ATTR_ID : AttrID = c"test-attrib".as_ptr() as *const u8;

	#[test]
	fn message_selftest() {
		// Rust'n'RefCount
		// Create a ComObject instance to wrap a box'ed Message (rc=0)
		let msg_id = "AwesomeMsg";
		let obj = ComObject::new(Message::with_id(msg_id));
		// Cast the ComObject to IMessage (rc=1)
		let msg : IMessage = obj.cast().expect("Message should implement IMessage");

		// Now we can use the unsafe interface methods
		let id_raw = unsafe { msg.getMessageID() };
		let expected_id_raw = StrConv::str_to_c_str_vec(msg_id);

		assert_eq!(StrConv::c_str_cmp(id_raw, expected_id_raw.as_ptr()), 0,
			"message should retain message identifier");

		// Rust'n'Lifetime
		// The 'raw_attribs' pointer must use at least the lifetime of the
		// interface reference 'attribs', so declare an initialize it here
		// and not in side from_raw_borrowed(...) or the surrounding unsafe block.
		// Using 'from_raw()' here would create an _owned_ interface, which is dropped
		// prematurely when 'attribs' goes out of scope. Later, when 'msg' goes out
		// of scope, it panics because of rc underflow.
		let raw_attribs = unsafe { msg.getAttributes() } as *mut std::ffi::c_void;
		let attribs : &IAttributeList = unsafe { IAttributeList::from_raw_borrowed(&raw_attribs) }
			.expect("IMessage::getAttributes() should provide a valid IAttributeList* to be borrowed");

		assert!(unsafe { attribs.setString(TEST_ATTR_ID, w!("This is my message to you-ooh-ooh").as_ptr()) }.is_ok(),
			"setting string-attribute should succeed");

		let mut buffer = [0u16, 64];
		assert!(unsafe { attribs.getString(TEST_ATTR_ID, buffer.as_mut_ptr(), (buffer.len() * std::mem::size_of::<u16>()) as u32) }.is_ok(),
			"retrieving string-attribute should succeed");
	}
}
