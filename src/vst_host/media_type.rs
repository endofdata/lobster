use com::AbiTransferable;

#[derive(Copy, Clone, Debug)]
pub enum MediaType
{
	Audio = 0,
	Event,
	NumMediaTypes
}

unsafe impl AbiTransferable for MediaType {
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