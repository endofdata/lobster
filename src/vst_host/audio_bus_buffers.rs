#[allow(dead_code)]
pub struct AudioBusBuffers {
	num_channels: i32,
	silence_flags: u64,
	// TODO: this is a union for f32 or f64
	channel_buffers: *mut ()
}