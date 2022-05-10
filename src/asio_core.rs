use com::sys::{
    CoCreateInstance, CLSCTX_INPROC_SERVER, CLSID, FAILED, HRESULT, IID,
};

use core::ffi::c_void;

pub struct ASIODriverInfo {
	pub asio_version : i32,			// currently, 2
	pub driver_version : i32,		// driver specific
	pub name: [u8; 32],
	pub error_message: [u8; 124],
	pub sys_ref: *const ()			// on input: system reference
									// (Windows: application main window handle, Mac & SGI: 0)
}

pub struct ASIOClockSource {
	pub index: i32,					// as used for ASIOSetClockSource()
	pub associated_channel: i32,	// for instance, S/PDIF or AES/EBU
	pub associated_group: i32,		// see channel groups (ASIOGetChannelInfo())
	pub is_current_source: i32,		// ASIOTrue if this is the current clock source
	pub name: [u8; 32]				// for user selection
}

pub struct ASIOChannelInfo {
	pub channel: i32,				// on input, channel index
	pub is_input: i32,				// on input
	pub is_active: i32,				// on exit
	pub channel_group: i32,			// dto
	pub sample_type: i32,			// dto
	pub name: [u8; 32]				// dto
}

pub struct ASIOBufferInfo {
	pub is_input: i32,				// on input:  ASIOTrue: input, else output
	pub channel_num: i32,			// on input:  channel index
	pub buffers: [*mut (); 2]		// on output: double buffer addresses
}

pub struct AsioTimeInfo {
	pub speed: f64,					// absolute speed (1. = nominal)
	pub system_time: i64,			// system time related to samplePosition, in nanoseconds
									// on mac, must be derived from Microseconds() (not UpTime()!)
									// on windows, must be derived from timeGetTime()
	pub sample_position: i64,
	pub sample_rate: f64,           // current rate
	pub flags: u32,					// (see below)
	pub reserved: [u8; 12]
}

pub struct ASIOTimeCode {       
	pub speed: f64,					// speed relation (fraction of nominal speed)
									// optional; set to 0. or 1. if not supported
	pub time_code_samples: i64,		// time in samples
	pub flags: u32,					// some information flags (see below)
	pub future: [u8; 64]
}

pub struct ASIOTime {				// both input/output
	pub reserved: [i32; 4],			// must be 0
	pub time_info: AsioTimeInfo,	// required
	pub time_code: ASIOTimeCode		// optional, evaluated if (timeCode.flags & kTcValid)
}

pub struct ASIOCallbacks
{
	pub buffer_switch: extern fn(double_buffer_index: i32, direct_process: ASIOBool),
		// bufferSwitch indicates that both input and output are to be processed.
		// the current buffer half index (0 for A, 1 for B) determines
		// - the output buffer that the host should start to fill. the other buffer
		//   will be passed to output hardware regardless of whether it got filled
		//   in time or not.
		// - the input buffer that is now filled with incoming data. Note that
		//   because of the synchronicity of i/o, the input always has at
		//   least one buffer latency in relation to the output.
		// directProcess suggests to the host whether it should immedeately
		// start processing (directProcess == ASIOTrue), or whether its process
		// should be deferred because the call comes from a very low level
		// (for instance, a high level priority interrupt), and direct processing
		// would cause timing instabilities for the rest of the system. If in doubt,
		// directProcess should be set to ASIOFalse.
		// Note: bufferSwitch may be called at interrupt time for highest efficiency.

	pub sample_rate_did_change: extern fn(sample_rate: f64),
		// gets called when the AudioStreamIO detects a sample rate change
		// If sample rate is unknown, 0 is passed (for instance, clock loss
		// when externally synchronized).

	pub asio_message: extern fn(selector: i32, value: i32, message: *const (), opt: *const f64),
		// generic callback for various purposes, see selectors below.
		// note this is only present if the asio version is 2 or higher

	pub buffer_switch_time_info: extern fn(params: *const ASIOTime, double_buffer_index: i32, direct_process: ASIOBool)
		// new callback with time info. makes ASIOGetSamplePosition() and various
		// calls to ASIOGetSampleRate obsolete,
		// and allows for timecode sync etc. to be preferred; will be used if
		// the driver calls asioMessage with selector kAsioSupportsTimeInfo.
}

pub type ASIOBool = i32;
pub type ASIOError = i32;

com::interfaces! {
    #[uuid("00000000-0000-0000-C000-000000000046")]
    pub unsafe interface IUnknown {
        pub fn QueryInterface(&self, riid: *const com::IID, ppv: *mut *mut ()) -> i32;
        pub fn AddRef(&self) -> u32;
        pub fn Release(&self) -> u32;
    }

	// This is generated by me, because Steinberg did not provide IID for IASIO
	#[uuid("250c2374-15d5-4425-a20b-942b85a6397f")]
	pub unsafe interface IASIO : IUnknown {		
		pub fn init(&self, sys_handle: *mut ()) -> ASIOBool;
		pub fn get_driver_name(&self, name: *mut u8);	
		pub fn get_driver_version(&self) -> i32;
		pub fn get_error_message(&self, text: *mut u8);	
		pub fn start(&self) -> ASIOError;
		pub fn stop(&self) -> ASIOError;
		pub fn get_channels(&self, num_input_channels: *mut i32, num_output_channels: *mut i32) -> ASIOError;
		pub fn get_latencies(&self, input_latency: *mut i32, output_latency: *mut i32) -> ASIOError;
		pub fn get_buffer_size(&self, min_size: *mut i32, max_size: *mut i32, preferred_size: *mut i32, granularity: *mut i32) -> ASIOError;
		pub fn can_sample_rate(&self, sample_rate: f64) -> ASIOError;
		pub fn get_sample_rate(&self, sample_rate: *mut f64) -> ASIOError;
		pub fn set_sample_rate(&self, sample_rate: f64) -> ASIOError;
		pub fn get_clock_sources(&self, clocks: *mut ASIOClockSource, num_sources: *mut i32) -> ASIOError;
		pub fn set_clock_source(&self, reference: i32) -> ASIOError;
		pub fn get_sample_position(&self, sample_pos: *mut i64, time_stamp: *mut i64) -> ASIOError;
		pub fn get_channel_info(&self, info: *mut ASIOChannelInfo) -> ASIOError;
		pub fn create_buffers(&self, buffer_infos: *mut ASIOBufferInfo, num_channels: i32, buffer_size: i32, callbacks: *const ASIOCallbacks) -> ASIOError;
		pub fn dispose_buffers(&self) -> ASIOError;
		pub fn control_panel(&self) -> ASIOError;
		pub fn future(&self, sel: i32, opt: *mut ()) -> ASIOError;
		pub fn output_ready(&self) -> ASIOError;
	}
}

pub fn create_device(class_id: &com::CLSID) -> Result<IASIO, HRESULT> {
	
	let mut instance : Option<IASIO> = None;

	println!("Creating ASIO device from GUID '{}'.", class_id);
	
	// Special handling for ASIO: class ID and interface ID are same
	let hr = unsafe {
		CoCreateInstance(class_id as *const CLSID, core::ptr::null_mut::<c_void>(), CLSCTX_INPROC_SERVER,
			class_id as *const IID, &mut instance as *mut _ as _)
	};

	if FAILED(hr) {
		return Err(hr);
	}

	let driver = instance.unwrap();

	return Ok(driver);
}
