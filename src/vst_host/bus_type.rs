use com::AbiTransferable;

#[derive(Copy, Clone, Debug)]
#[allow(dead_code)]
pub enum BusType
{
	Main = 0,
	Aux
}

unsafe impl AbiTransferable for BusType {
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