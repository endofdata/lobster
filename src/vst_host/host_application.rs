use windows::{
	core::{implement, ComObject, Interface, GUID, HRESULT}, Win32::Foundation::{E_NOINTERFACE, S_OK}
};

use crate::vst_host::{attrib_list::AttributeList, message::Message};
use crate::vst_host::{IAttributeList, IMessage};
use crate::vst_host::{host::Host, IHostApplication_Impl, String128};
use super::IHostApplication;

#[implement(IHostApplication)]
pub struct HostApplication<'a>(&'a Host);

impl <'a> HostApplication<'a> {
	pub fn new(host: &'a Host) -> Self {
		Self(host)
	}
}

impl<'a> IHostApplication_Impl for HostApplication_Impl<'a> {
	unsafe fn getName(&self, name: String128) -> i32 {
		if name != std::ptr::null_mut() {
			let host_name = self.0.get_name().encode_utf16();
			let count = usize::min(self.0.get_name().chars().count(), super::STRING_128_SIZE - 1);
			let mut target = name;
			for c in host_name.take(count) {
				unsafe {
					*target = c;
					target = target.add(1);
				}
			}
			count as i32
		}
		else {
			0
		}
	}

	unsafe fn createInstance(&self, cid: *const GUID, iid: *const GUID, ppv: *mut *const std::ffi::c_void) -> HRESULT {
		let cid : GUID = unsafe { *cid };
		let iid : GUID = unsafe { *iid };
		unsafe { *ppv = std::ptr::null_mut()};

		if iid == IMessage::IID {
			if let Ok(message) = ComObject::new(Message::new()).cast::<IMessage>() {
				unsafe { *ppv = message.into_raw() as *mut std::ffi::c_void };
				S_OK
			}
			else {
				E_NOINTERFACE
			}
		}
		else if iid == IAttributeList::IID {
			if let Ok(attrib_list) = ComObject::new(AttributeList::new()).cast::<IAttributeList>() {
				unsafe { *ppv = attrib_list.into_raw() as *mut std::ffi::c_void };
				S_OK
			}
			else {
				E_NOINTERFACE
			}
		}
		else {
			eprintln!("Request for unsupported instance: class id '{:?}', interface id '{:?}'.", cid, iid);
			E_NOINTERFACE
		}
	}
}