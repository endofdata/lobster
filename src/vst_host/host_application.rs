use windows::Win32::Foundation::E_NOINTERFACE;
use windows::core::{implement, GUID, HRESULT};
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

	unsafe fn createInstance(&self, cid: *const GUID, iid: *const GUID, ppv: *mut *mut std::ffi::c_void) -> HRESULT {
		eprintln!("Request for unsupported instance: class id '{:?}', interface id '{:?}'.", cid, iid);
		unsafe { *ppv = std::ptr::null_mut()};

		E_NOINTERFACE
	}
}