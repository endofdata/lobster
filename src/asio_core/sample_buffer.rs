const MAX_I32_VALUE: f64 = 2147483647.0f64;
//const PAN_LEFT: f64 = -1.0f64;
//const PAN_RIGHT: f64 = 1.0f64;

pub trait SampleConvert {
	type Sample;

	fn from_native(self) -> f64;
	fn to_native(sample: f64) -> Self::Sample;
}

impl SampleConvert for i32 {
	type Sample = i32;

	fn from_native(self) -> f64 {
		(self as f64) / MAX_I32_VALUE
	}

	fn to_native(sample: f64) -> Self::Sample {
		(sample * MAX_I32_VALUE) as Self::Sample
	}
}

impl SampleConvert for f64 {
	type Sample = f64;

	fn from_native(self) -> f64 {
		self
	}

	fn to_native(sample: f64) -> Self::Sample {
		sample
	}
}
