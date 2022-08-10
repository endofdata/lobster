use com::AbiTransferable;

type SampleRate = f64;

#[derive(Copy, Clone, Debug)]
#[allow(dead_code)]
pub struct ProcessSetup
{
	process_mode: i32,
	symbolic_sample_size: i32,
	max_samples_per_block: i32,
	sample_rate: SampleRate
}

unsafe impl AbiTransferable for ProcessSetup {
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
