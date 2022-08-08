use com::AbiTransferable;

#[derive(Copy, Clone, Debug)]
#[allow(dead_code)]
pub enum BusFlags
{
	DefaultActive = 1 << 0,
	IsControlVoltage = 1 << 1
}

unsafe impl AbiTransferable for BusFlags {
	type Abi = i32;

	fn set_abi(&mut self) -> *mut Self::Abi {
		std::ptr::addr_of_mut!(*self) as *mut i32
	}

	fn get_abi(&self) -> Self::Abi {
		unsafe {
			*(std::ptr::addr_of!(self) as *const i32)
		}
	}
}
