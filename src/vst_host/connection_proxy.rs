use std::cell::RefCell;

use crate::vst_host::{IConnectionPoint, IConnectionPoint_Impl, IMessage};
use windows::Win32::Foundation::{E_INVALIDARG, S_FALSE, S_OK};
use windows_core::{implement, Interface, HRESULT};

#[implement(IConnectionPoint)]
pub struct ConnectionProxy
{
	src: RefCell<IConnectionPoint>,
	dst: RefCell<Option<IConnectionPoint>>,
}

impl ConnectionProxy {
	pub fn new(src: IConnectionPoint) -> ConnectionProxy {
		ConnectionProxy { src: RefCell::new(src), dst: RefCell::new(None) }
	}
}

impl Drop for ConnectionProxy {
	fn drop(&mut self) {
		println!("Dropping connection proxy");
	}
}

fn iface_from_raw<'a, T: Interface>(raw: *const T) -> Option<T> {
	let raw_mut_void = raw as *mut std::ffi::c_void;
	unsafe { T::from_raw_borrowed(&raw_mut_void) }
		.and_then(|iface| Some(iface.clone()))
}

impl IConnectionPoint_Impl for ConnectionProxy_Impl {
	unsafe fn connect(&self, dst: *const IConnectionPoint) -> HRESULT {
		if dst == std::ptr::null() {
			return E_INVALIDARG;
		}
		let dst_as_mut_void = dst as *mut std::ffi::c_void;
		*self.dst.borrow_mut() = Some(unsafe { IConnectionPoint::from_raw_borrowed(&dst_as_mut_void).unwrap() }.clone());
		S_OK
	}

	unsafe fn disconnect(&self, _dst: *const IConnectionPoint) -> HRESULT {
		// TODO: check if current connection is same as dst
		*self.dst.borrow_mut() = None;
		S_OK
	}

	unsafe fn notify(&self,msg: *const IMessage) -> HRESULT {
		// TODO: ignore message if not in main UI thread
		match self.dst.borrow().as_ref() {
			Some(dst) => unsafe { dst.notify(msg) },
			_ => S_FALSE
		}
	}
}

#[cfg(test)]
mod test {
	use std::cell::RefCell;
	use std::collections::HashMap;

	use windows::Win32::Foundation::{E_FAIL, E_INVALIDARG, S_OK};
	use windows::core::{implement, Interface, HRESULT};
	use windows_core::{w, ComObject};
	use crate::vst_host::str_conv::StrConv;
	use crate::vst_host::{AttrID, FIDString, IAttributeList, IAttributeList_Impl, IConnectionPoint, IConnectionPoint_Impl, IMessage, IMessage_Impl};
	use super::ConnectionProxy;

	const MAX_MSG_LEN : usize = 128;
	const MAX_ATTRKEY_LEN: usize = 64;
	const TEST_ATTR_ID : AttrID = c"test-attrib".as_ptr() as *const u8;

	#[derive(Eq, Hash, PartialEq)]
	struct AttrKey([u8;MAX_ATTRKEY_LEN]);

	impl From<AttrID> for AttrKey {
		fn from(value: AttrID) -> Self {
			let mut buffer = [0u8; MAX_ATTRKEY_LEN];
			StrConv::c_str_copy(value, &mut buffer);
			AttrKey(buffer)
		}
	}

	#[derive(Clone)]
	enum AttrValue {
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
	struct AttributeList {
		values: RefCell<HashMap<AttrKey, AttrValue>>
	}

	impl AttributeList {
		pub fn new() -> Self {
			Self {
				values: RefCell::new(HashMap::<AttrKey, AttrValue>::new())
			}
		}

		fn set_attribute(&self, id: &AttrID, value: AttrValue) -> HRESULT {
			let _opt_old_value = self.values.borrow_mut().insert((*id).into(), value);
			S_OK
		}

		fn get_attribute(&self, id: &AttrID) -> Option<AttrValue> {
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

	#[implement(IMessage)]
	struct Message {
		id: RefCell<Vec<u8>>,
		attribs: IAttributeList
	}

	impl Message {
		pub fn new(id: &str) -> Self {
			Self {
				id: RefCell::new(StrConv::str_to_c_str_vec(id)),
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
			self.id.borrow().as_ptr()
		}

		unsafe fn setMessageID(&self, id: FIDString) {
			*self.id.borrow_mut() = StrConv::c_str_to_vec(id)
		}

		unsafe fn getAttributes(&self) ->  *const IAttributeList {
			self.attribs.as_raw() as *const IAttributeList
		}
	}

	#[implement(IConnectionPoint)]
	struct ConnectionPoint {
		name: String,
		dsts: RefCell<Vec<IConnectionPoint>>
	}

	impl ConnectionPoint {
		fn new(name: &str) -> ConnectionPoint {
			ConnectionPoint { name: name.into(), dsts: RefCell::new(Vec::<IConnectionPoint>::new()) }
		}
	}

	impl Drop for ConnectionPoint {
		fn drop(&mut self) {
			println!("Dropping connection point '{}'", self.name);
		}
	}

	impl IConnectionPoint_Impl for ConnectionPoint_Impl {
		unsafe fn connect(&self, dst: *const IConnectionPoint) -> HRESULT {
			let dst_as_void = dst as *mut std::ffi::c_void;
			match unsafe { IConnectionPoint::from_raw_borrowed(&dst_as_void) } {
				None => E_INVALIDARG,
				Some(dst_ref) => {
					let dsts : &mut Vec<IConnectionPoint> = &mut *self.dsts.borrow_mut();

					if dsts.contains(dst_ref) {
						E_FAIL
					}
					else {
						dsts.push(dst_ref.clone());
						S_OK
					}
				}
			}
		}

		unsafe fn disconnect(&self, dst: *const IConnectionPoint) -> HRESULT {
			let dst_as_mut_void = dst as *mut std::ffi::c_void;
			match unsafe { IConnectionPoint::from_raw_borrowed(&dst_as_mut_void) } {
				None => E_INVALIDARG,
				Some(dst_ref) => {
					let dsts : &mut Vec<IConnectionPoint> = &mut *self.dsts.borrow_mut();

					if let Some(index) = dsts.iter().position(|d| d == dst_ref) {
						dsts.remove(index);
						S_OK
					}
					else {
						E_FAIL
					}
				}
			}
		}

		unsafe fn notify(&self, msg: *const IMessage) -> HRESULT {
			let msg_as_mut_void = msg as *mut std::ffi::c_void;
			match unsafe { IMessage::from_raw_borrowed(&msg_as_mut_void) } {
				None => E_INVALIDARG,
				Some(msg_ref) => {
					let attribs_as_mut_void = unsafe { msg_ref.getAttributes() } as *mut std::ffi::c_void;
					match unsafe { IAttributeList::from_raw_borrowed(&attribs_as_mut_void) } {
						None => E_INVALIDARG,
						Some(attribs_ref) => {
							match unsafe {
								let mut buffer = [0u16; MAX_MSG_LEN];
								attribs_ref.getString(TEST_ATTR_ID, buffer.as_mut_ptr(), MAX_MSG_LEN as u32)
									.and_then(|| String::from_utf16(&buffer).or_else(|e| Ok(e.to_string())))
							} {
								Ok(text) => {
									let msg_id = StrConv::c_str_to_string(unsafe { msg_ref.getMessageID() });
								 	println!("IConnectionPoint '{}' sends message id '{}', text: '{}' to {} destination(s)",
										self.name, msg_id, text, (*self.dsts.borrow()).len())
								},
								Err(e) => eprintln!("Notification message should contain string attribute with id TEST_ATTR_ID. Error: {}", e)
							}

							for dst in &*self.dsts.borrow() {
								let hr = unsafe { dst.notify(msg) };
								if hr.is_err() {
									return hr;
								}
							}
							S_OK
						}
					}
				}
			}

		}

	}

	fn send_test_msg(to: &IConnectionPoint) {
		let msg : IMessage = ComObject::new(Message::new("gulliver")).cast().unwrap();
		let raw_attribs = unsafe { msg.getAttributes() as *mut std::ffi::c_void };
		let attribs : &IAttributeList = unsafe { IAttributeList::from_raw_borrowed(&raw_attribs).unwrap() };

		assert!(unsafe { attribs.setString(TEST_ATTR_ID, w!("is still travelling").as_ptr()) }.is_ok(),
			"adding test message attribute with IAttribList::setString() should be successful");

		assert!(unsafe { to.notify(msg.as_raw() as *const IMessage) }.is_ok(),
			"source should successful send notification.");
	}

	#[test]
	fn message_selftest() {
		// Rust'n'RefCount
		// Create a ComObject instance to wrap a box'ed Message (rc=0)
		let msg_id = "AwesomeMsg";
		let obj = ComObject::new(Message::new(msg_id));
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

	#[test]
	fn connection_point_selftest() {
		let src : IConnectionPoint = ComObject::new(ConnectionPoint::new("source")).cast().unwrap();
		let dst : IConnectionPoint = ComObject::new(ConnectionPoint::new("destination")).cast().unwrap();

		assert!(unsafe { src.connect(dst.as_raw() as *const IConnectionPoint) }.is_ok(),
			"source should accept connection to destination");

		send_test_msg(&src);

		assert!(unsafe { src.disconnect(dst.as_raw() as *const IConnectionPoint) }.is_ok(),
			"source should successfully disconnect from destination");
	}

	#[test]
	fn connect_proxies() {
		let test_point = ConnectionPoint::new("test");
		let proxy = ConnectionProxy::new(test_point.into());

		send_test_msg(&*proxy.src.borrow());
	}
}