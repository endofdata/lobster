#[derive(Copy, Clone, Debug)]
pub enum ComponentFlags {
	NoValue = 0,
	// TODO: Validate internal enum values
	Distributable = 1,
	SimpleModeSupported = 1 << 1
}