use std::cell::RefCell;

use crate::vst_host::{thread_check::ThreadCheck, IConnectionPoint, IConnectionPoint_Impl, IMessage};
use windows::Win32::Foundation::{E_INVALIDARG, S_FALSE};
use windows_core::{implement, ComObjectInterface, Interface, InterfaceRef, HRESULT};

#[implement(IConnectionPoint)]
pub struct ConnectionProxy
{
	src: RefCell<IConnectionPoint>,
	dst: RefCell<Option<IConnectionPoint>>,
	thread_check: ThreadCheck,
}

impl ConnectionProxy {
	pub fn new(src: IConnectionPoint, thread_check: ThreadCheck) -> ConnectionProxy {
		println!("New connection proxy");
		ConnectionProxy { src: RefCell::new(src), dst: RefCell::new(None), thread_check }
	}

	pub fn get_src(&self) -> IConnectionPoint {
		self.src.borrow().clone()
	}

	pub fn get_dst(&self) -> Option<IConnectionPoint> {
		self.dst.borrow().clone()
	}
}

impl Drop for ConnectionProxy {
	fn drop(&mut self) {
		println!("Dropping connection proxy");
	}
}

impl IConnectionPoint_Impl for ConnectionProxy_Impl {
	unsafe fn connect(&self, other: *const IConnectionPoint) -> HRESULT {
		if other == std::ptr::null() {
			return E_INVALIDARG;
		}
		if self.dst.borrow().is_some() {
			return S_FALSE;
		}

		let dst_as_mut_void = other as *mut std::ffi::c_void;
		*self.dst.borrow_mut() = Some(unsafe { IConnectionPoint::from_raw_borrowed(&dst_as_mut_void).unwrap() }.clone());

		let self_as_iface : InterfaceRef<IConnectionPoint> = self.as_interface_ref();
		let hr = unsafe { self.src.borrow().connect(std::mem::transmute_copy(&self_as_iface)) };

		if hr.is_err() {
			*self.dst.borrow_mut() = None;
		}

		hr
	}

	unsafe fn disconnect(&self, other: *const IConnectionPoint) -> HRESULT {
		if other == std::ptr::null() {
			return E_INVALIDARG;
		}

		let hr = match &*self.dst.borrow() {
			Some(dst) => {
				if other != unsafe { std::mem::transmute_copy(dst) } {
					E_INVALIDARG
				}
				else {
					let self_as_iface : InterfaceRef<IConnectionPoint> = self.as_interface_ref();
					unsafe { self.src.borrow().disconnect(std::mem::transmute_copy(&self_as_iface)) }
				}
			},
			None => E_INVALIDARG
		};

		if hr.is_ok() {
			*self.dst.borrow_mut() = None;
		}
		hr
	}

	unsafe fn notify(&self,msg: *const IMessage) -> HRESULT {
		self.thread_check.with_expected(|| {
			match self.dst.borrow().as_ref() {
				Some(dst) => unsafe { dst.notify(msg) },
				_ => S_FALSE
			}
		}).unwrap_or(S_FALSE)
	}
}

#[cfg(test)]
mod test {
	use std::cell::RefCell;

	use windows::{
		Win32::Foundation::{E_FAIL, E_INVALIDARG, S_OK},
		core::{implement, Interface, InterfaceRef, HRESULT, w, ComObject}
	};

	use super::{
		ConnectionProxy,
		super::{
			thread_check::ThreadCheck,
			message::Message,
			AttrID, IAttributeList, IConnectionPoint, IConnectionPoint_Impl, IMessage,
		}
	};

	use crate::os::StrConv;

	const TEST_ATTR_ID : AttrID = c"test-attrib".as_ptr() as *const u8;
	const MAX_MSG_LEN : usize = 128;


	#[implement(IConnectionPoint)]
	struct ConnectionPoint {
		name: String,
		dsts: RefCell<Vec<IConnectionPoint>>
	}

	impl ConnectionPoint {
		fn new(name: &str) -> ConnectionPoint {
			println!("New connection point '{}'", name);
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
		let msg : IMessage = ComObject::new(Message::with_id("gulliver")).cast().unwrap();
		let raw_attribs : *mut std::ffi::c_void = unsafe { std::mem::transmute_copy(&msg.getAttributes()) };
		let attribs : &IAttributeList = unsafe { IAttributeList::from_raw_borrowed(&raw_attribs).unwrap() };

		assert!(unsafe { attribs.setString(TEST_ATTR_ID, w!("is still travelling").as_ptr()) }.is_ok(),
			"adding test message attribute with IAttribList::setString() should be successful");

		assert!(unsafe { to.notify(std::mem::transmute_copy(&msg)) }.is_ok(),
			"source should successful send notification.");
	}


	#[test]
	fn connection_point_selftest() {
		let src : IConnectionPoint = ComObject::new(ConnectionPoint::new("source")).cast().unwrap();
		let dst : IConnectionPoint = ComObject::new(ConnectionPoint::new("destination")).cast().unwrap();

		assert!(unsafe { src.connect(std::mem::transmute_copy(&dst)) }.is_ok(),
			"source should accept connection to destination");

		send_test_msg(&src);

		assert!(unsafe { src.disconnect(std::mem::transmute_copy(&dst)) }.is_ok(),
			"source should successfully disconnect from destination");
	}

	#[test]
	fn connect_proxies() {
		println!("begin scope");
		{
			let cp_src = ComObject::new(ConnectionPoint::new("test-src"));
			let cp_dst = ComObject::new(ConnectionPoint::new("test-dst"));
			let proxy = ComObject::new(ConnectionProxy::new(cp_src.into_interface(), ThreadCheck::for_current_thread()));
			let cp_proxy : InterfaceRef<IConnectionPoint> = proxy.as_interface();

			let dst_raw: IConnectionPoint = cp_dst.cast::<IConnectionPoint>().unwrap();

			assert!(unsafe { cp_proxy.connect(std::mem::transmute_copy(&dst_raw)) }.is_ok(),
				"connection proxy should accept conntection to destination");

			send_test_msg(&cp_proxy);

			// IMPORTANT: current implementation leaks memory if not explicitly disconnected!
			assert!(unsafe { cp_proxy.disconnect(std::mem::transmute_copy(&dst_raw)) }.is_ok(),
				"connection proxy should disconnect from connected destination");
		}
		println!("scope terminated");
	}
}