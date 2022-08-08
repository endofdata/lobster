use com::AbiTransferable;

#[derive(Copy, Clone, Debug)]
pub enum SeekMode {
	IBSeekSet = 0,
	IBSeekCur,
	IBSeekEnd
}

unsafe impl AbiTransferable for SeekMode {
	type Abi = i32;

	fn set_abi(&mut self) -> *mut Self::Abi {
		unsafe {
			std::ptr::addr_of_mut!(self) as *mut i32
		}
	}

	fn get_abi(&self) -> Self::Abi {
		unsafe {
			*(std::ptr::addr_of!(self) as *const i32)
		}
	}
}
