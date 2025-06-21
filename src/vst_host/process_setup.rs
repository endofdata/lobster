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
