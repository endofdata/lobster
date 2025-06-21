#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub enum IoMode {
	Simple = 0,
	Advanced,
	OfflineProcessing
}
