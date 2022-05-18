pub mod sample_buffer;
pub mod asio_device;

use com::sys::{
    CoCreateInstance, CLSCTX_INPROC_SERVER, CLSID, FAILED, HRESULT, IID,
};

use std::fmt;
use core::ffi::c_void;

#[repr(i32)]
#[derive(Copy, Clone, Debug, PartialEq)]
#[allow(dead_code)]
pub enum ASIOBool {
	False = 0,
	True = 1
}

#[repr(i32)]
#[derive(Copy, Clone, Debug, PartialEq)]
#[allow(dead_code)]
pub enum ASIOError {
	Ok = 0,             	// This value will be returned whenever the call succeeded
	Success = 0x3f4847a0,	// unique success return value for ASIOFuture calls
	NotPresent = -1000, 	// hardware input or output is not present or available
	HWMalfunction,      	// hardware is malfunctioning (can be returned by any ASIO function)
	InvalidParameter,   	// input parameter invalid
	InvalidMode,        	// hardware is in a bad mode or used in a bad mode
	SPNotAdvancing,     	// hardware is not running when sample position is inquired
	NoClock,            	// sample clock or rate cannot be determined or is not present
	NoMemory            	// not enough memory for completing the request
}

#[repr(i32)]
#[derive(Copy, Clone, Debug, PartialEq)]
#[allow(dead_code)]
pub enum ASIOSampleType {
	Int16MSB   = 0,
	Int24MSB   = 1,		// used for 20 bits as well
	Int32MSB   = 2,
	Float32MSB = 3,		// IEEE 754 32 bit float
	Float64MSB = 4,		// IEEE 754 64 bit double float

	// these are used for 32 bit data buffer, with different alignment of the data inside
	// 32 bit PCI bus systems can be more easily used with these
	Int32MSB16 = 8,		// 32 bit data with 16 bit alignment
	Int32MSB18 = 9,		// 32 bit data with 18 bit alignment
	Int32MSB20 = 10,		// 32 bit data with 20 bit alignment
	Int32MSB24 = 11,		// 32 bit data with 24 bit alignment
	
	Int16LSB   = 16,
	Int24LSB   = 17,		// used for 20 bits as well
	Int32LSB   = 18,
	Float32LSB = 19,		// IEEE 754 32 bit float, as found on Intel x86 architecture
	Float64LSB = 20, 		// IEEE 754 64 bit double float, as found on Intel x86 architecture

	// these are used for 32 bit data buffer, with different alignment of the data inside
	// 32 bit PCI bus systems can more easily used with these
	Int32LSB16 = 24,		// 32 bit data with 18 bit alignment
	Int32LSB18 = 25,		// 32 bit data with 18 bit alignment
	Int32LSB20 = 26,		// 32 bit data with 20 bit alignment
	Int32LSB24 = 27,		// 32 bit data with 24 bit alignment

	//	ASIO DSD format.
	DSDInt8LSB1   = 32,		// DSD 1 bit data, 8 samples per byte. First sample in Least significant bit.
	DSDInt8MSB1   = 33,		// DSD 1 bit data, 8 samples per byte. First sample in Most significant bit.
	DSDInt8NER8	= 40,		// DSD 8 bit data, 1 sample per byte. No Endianness required.

	LastEntry
}

impl ASIOSampleType {
	pub fn size_in_bytes(&self) -> usize {
		match self {
			ASIOSampleType::Int16MSB   => 2,
			ASIOSampleType::Int24MSB   => 3,		// used for 20 bits as well
			ASIOSampleType::Int32MSB   => 4,
			ASIOSampleType::Float32MSB => 4,		// IEEE 754 32 bit float
			ASIOSampleType::Float64MSB => 8,		// IEEE 754 64 bit double float
		
			ASIOSampleType::Int32MSB16 => 4,		// 32 bit data with 16 bit alignment
			ASIOSampleType::Int32MSB18 => 4,		// 32 bit data with 18 bit alignment
			ASIOSampleType::Int32MSB20 => 4,		// 32 bit data with 20 bit alignment
			ASIOSampleType::Int32MSB24 => 4,		// 32 bit data with 24 bit alignment
			
			ASIOSampleType::Int16LSB   => 2,
			ASIOSampleType::Int24LSB   => 3,		// used for 20 bits as well
			ASIOSampleType::Int32LSB   => 4,
			ASIOSampleType::Float32LSB => 4,		// IEEE 754 32 bit float, as found on Intel x86 architecture
			ASIOSampleType::Float64LSB => 8, 		// IEEE 754 64 bit double float, as found on Intel x86 architecture
		
			ASIOSampleType::Int32LSB16 => 2,		// 32 bit data with 18 bit alignment
			ASIOSampleType::Int32LSB18 => 4,		// 32 bit data with 18 bit alignment
			ASIOSampleType::Int32LSB20 => 4,		// 32 bit data with 20 bit alignment
			ASIOSampleType::Int32LSB24 => 4,		// 32 bit data with 24 bit alignment
		
			//	ASIO DSD format.
			ASIOSampleType::DSDInt8LSB1 => 1,		// DSD 1 bit data, 8 samples per byte. First sample in Least significant bit.
			ASIOSampleType::DSDInt8MSB1 => 1,		// DSD 1 bit data, 8 samples per byte. First sample in Most significant bit.
			ASIOSampleType::DSDInt8NER8	=> 1,		// DSD 8 bit data, 1 sample per byte. No Endianness required.
			_ => panic!("Invalid sample type: '{:?}'", self)
		}
	}
}

#[allow(dead_code)]
pub fn sample_count(sample_rate: f64, ms: f64) -> f64 {
	(sample_rate * ms + 1.0) / 1000.0
}

#[allow(dead_code)]
pub fn buffer_size(sample_rate: f64, ms: f64, sample_type: ASIOSampleType) -> i64 {
	let sample_size = sample_type.size_in_bytes() as f64;

	(sample_count(sample_rate, ms) * sample_size) as i64
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct DriverInfo {
	pub asio_version : i32,			// currently, 2
	pub driver_version : i32,		// driver specific
	pub name: [u8; 32],
	pub error_message: [u8; 124],
	pub sys_ref: *const ()			// on input: system reference
									// (Windows: application main window handle, Mac & SGI: 0)
}

#[repr(i32)]
#[derive(Copy, Clone, PartialEq)]
#[allow(dead_code)]
pub enum MessageSelector
{
	SelectorSupported = 1,	// selector in <value>, returns 1L if supported,
								// 0 otherwise
    EngineVersion,			// returns engine (host) asio implementation version,
								// 2 or higher
	ResetRequest,			// request driver reset. if accepted, this
								// will close the driver (ASIO_Exit() ) and
								// re-open it again (ASIO_Init() etc). some
								// drivers need to reconfigure for instance
								// when the sample rate changes, or some basic
								// changes have been made in ASIO_ControlPanel().
								// returns 1L; note the request is merely passed
								// to the application, there is no way to determine
								// if it gets accepted at this time (but it usually
								// will be).
	BufferSizeChange,		// not yet supported, will currently always return 0L.
								// for now, use kAsioResetRequest instead.
								// once implemented, the new buffer size is expected
								// in <value>, and on success returns 1L
	ResyncRequest,			// the driver went out of sync, such that
								// the timestamp is no longer valid. this
								// is a request to re-start the engine and
								// slave devices (sequencer). returns 1 for ok,
								// 0 if not supported.
	LatenciesChanged, 		// the drivers latencies have changed. The engine
								// will refetch the latencies.
	SupportsTimeInfo,		// if host returns true here, it will expect the
								// callback bufferSwitchTimeInfo to be called instead
								// of bufferSwitch
	SupportsTimeCode,		// 
	MMCCommand,			// unused - value: number of commands, message points to mmc commands
	SupportsInputMonitor,	// kAsioSupportsXXX return 1 if host supports this
	SupportsInputGain,     // unused and undefined
	SupportsInputMeter,    // unused and undefined
	SupportsOutputGain,    // unused and undefined
	SupportsOutputMeter,   // unused and undefined
	Overload,              // driver detected an overload

	NumMessageSelectors
}

unsafe impl com::AbiTransferable for MessageSelector {
    type Abi = Self;
    fn get_abi(&self) -> Self::Abi {
        *self
    }
    fn set_abi(&mut self) -> *mut Self::Abi {
        self as *mut Self::Abi
    }
}


#[repr(i32)]
#[derive(Copy, Clone, Debug, PartialEq)]
#[allow(dead_code)]
pub enum FutureSelector
{
	EnableTimeCodeRead = 1,		// no arguments
	DisableTimeCodeRead,		// no arguments
	SetInputMonitor,			// ASIOInputMonitor* in params
	Transport,					// ASIOTransportParameters* in params
	SetInputGain,				// ASIOChannelControls* in params, apply gain
	GetInputMeter,				// ASIOChannelControls* in params, fill meter
	SetOutputGain,				// ASIOChannelControls* in params, apply gain
	GetOutputMeter,				// ASIOChannelControls* in params, fill meter
	CanInputMonitor,			// no arguments for kAsioCanXXX selectors
	CanTimeInfo,
	CanTimeCode,
	CanTransport,
	CanInputGain,
	CanInputMeter,
	CanOutputGain,
	CanOutputMeter,
	OptionalOne,
	
	//	DSD support
	//	The following extensions are required to allow switching
	//	and control of the DSD subsystem.
	SetIoFormat					= 0x23111961,		/* ASIOIoFormat * in params.			*/
	GetIoFormat					= 0x23111983,		/* ASIOIoFormat * in params.			*/
	CanDoIoFormat				= 0x23112004,		/* ASIOIoFormat * in params.			*/
	
	// Extension for drop out detection
	CanReportOverload			= 0x24042012,	/* return ASE_SUCCESS if driver can detect and report overloads */
	
	GetInternalBufferSamples	= 0x25042012	/* ASIOInternalBufferInfo * in params. Deliver size of driver internal buffering, return ASE_SUCCESS if supported */
}

unsafe impl com::AbiTransferable for FutureSelector {
    type Abi = Self;
    fn get_abi(&self) -> Self::Abi {
        *self
    }
    fn set_abi(&mut self) -> *mut Self::Abi {
        self as *mut Self::Abi
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ClockSource {
	pub index: i32,						// as used for ASIOSetClockSource()
	pub associated_channel: i32,		// for instance, S/PDIF or AES/EBU
	pub associated_group: i32,			// see channel groups (ASIOGetChannelInfo())
	pub is_current_source: ASIOBool,	// ASIOTrue if this is the current clock source
	pub name: [u8; 32]					// for user selection
}

impl ClockSource {
	pub const fn new() -> ClockSource {
		ClockSource {
			index: 0,
			associated_channel: 0,
			associated_group: 0,
			is_current_source: ASIOBool::False,
			name: [0u8; 32]
		}
	}
}

impl fmt::Debug for ClockSource {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let trimmed_vec = self.name.iter().take_while(|c| **c != 0u8).cloned().collect();
		let name = String::from_utf8(trimmed_vec).expect("ClockSource.name is utf-8");

		f.debug_struct("ClockSource")
			.field("index", &self.index)
			.field("associated_channel", &self.associated_channel)
			.field("associated_group", &self.associated_group)
			.field("is_current_source", &self.is_current_source)
			.field("name", &name)
			.finish()
	}
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ChannelInfo {
	pub channel: i32,					// on input, channel index
	pub is_input: ASIOBool,				// on input
	pub is_active: ASIOBool,			// on exit
	pub channel_group: i32,				// dto
	pub sample_type: ASIOSampleType,	// dto
	pub name: [u8; 32]					// dto
}

impl ChannelInfo {
	pub const fn new() -> ChannelInfo {
		ChannelInfo {
			channel: 0,
			is_input: ASIOBool::False,
			is_active: ASIOBool::False,
			channel_group: 0,
			sample_type: ASIOSampleType::LastEntry,
			name: [0u8; 32]
		}
	}

	pub const fn new_for(is_input: ASIOBool, channel: i32) -> ChannelInfo {
		let mut channel_info = ChannelInfo::new();
		channel_info.channel = channel;
		channel_info.is_input = is_input;
		channel_info
	}
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct BufferInfo {
	pub is_input: ASIOBool,			// on input:  ASIOTrue: input, else output
	pub channel_num: i32,			// on input:  channel index
	pub buffers: [*mut (); 2]		// on output: double buffer addresses
}

impl BufferInfo {
	pub fn new() -> BufferInfo {
		BufferInfo {
			is_input: ASIOBool::False,
			channel_num: 0,
			buffers: [core::ptr::null_mut::<()>(); 2]
		}
	}
}

#[repr(u32)]
#[derive(Copy, Clone, PartialEq)]
pub enum TimeInfoFlags
{
	None				   = 0,
	SystemTimeValid        = 1,            // must always be valid
	SamplePositionValid    = 1 << 1,       // must always be valid
	SampleRateValid        = 1 << 2,
	SpeedValid             = 1 << 3,
	
	SampleRateChanged      = 1 << 4,
	ClockSourceChanged     = 1 << 5
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct TimeInfo {
	pub speed: f64,					// absolute speed (1. = nominal)
	pub system_time: i64,			// system time related to samplePosition, in nanoseconds
									// on mac, must be derived from Microseconds() (not UpTime()!)
									// on windows, must be derived from timeGetTime()
	pub sample_position: i64,
	pub sample_rate: f64,           // current rate
	pub flags: TimeInfoFlags,	// (see above)
	pub reserved: [u8; 12]
}

impl TimeInfo {
	pub const fn new() -> TimeInfo {
		TimeInfo {
			speed: 0.0,
			system_time: 0,
			sample_position: 0,
			sample_rate: 0.0,
			flags: TimeInfoFlags::None,
			reserved: [0u8; 12]
		}
	}
}

impl fmt::Debug for TimeInfo {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("TimeInfo")
			.field("speed", &self.speed)
			.field("system_time", &self.system_time)
			.field("sample_position", &self.sample_position)
			.field("sample_rate", &self.sample_rate)
			.field("flags", &(self.flags as i32))
			.finish()
	}
}

#[repr(u32)]
#[derive(Copy, Clone, PartialEq)]
pub enum TimeCodeFlags
{
	None				   = 0,
	TcValid                = 1,
	TcRunning              = 1 << 1,
	TcReverse              = 1 << 2,
	TcOnspeed              = 1 << 3,
	TcStill                = 1 << 4,
	
	TcSpeedValid           = 1 << 8
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct TimeCode {       
	pub speed: f64,					// speed relation (fraction of nominal speed)
									// optional; set to 0. or 1. if not supported
	pub time_code_samples: i64,		// time in samples
	pub flags: TimeCodeFlags,		// some information flags (see above)
	pub future: [u8; 64]
}

impl TimeCode {
	pub const fn new() -> TimeCode {
		TimeCode {
			speed: 0.0,
			time_code_samples: 0,
			flags: TimeCodeFlags::None,
			future: [0u8; 64]
		}
	}
}

impl fmt::Debug for TimeCode {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("TimeCode")
			.field("speed", &self.speed)
			.field("time_code_samples", &self.time_code_samples)
			.field("flags", &(self.flags as i32))
			.finish()
	}
}

#[repr(C)]
#[derive(Clone)]
pub struct Time {					// both input/output
	pub reserved: [i32; 4],			// must be 0
	pub time_info: TimeInfo,		// required
	pub time_code: TimeCode			// optional, evaluated if (timeCode.flags & kTcValid)
}

impl Time {
	pub const fn new() -> Time {
		Time {
			reserved: [0; 4],
			time_info: TimeInfo::new(),
			time_code: TimeCode::new()
		}
	}
}


impl fmt::Debug for Time {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("Time")
			.field("time_info", &self.time_info)
			.field("time_code", &self.time_code)
			.finish()
	}
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Callbacks
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

	pub asio_message: extern fn(selector: MessageSelector, value: i32, message: *mut (), opt: *const f64) -> i32,
		// generic callback for various purposes, see selectors below.
		// note this is only present if the asio version is 2 or higher

	pub buffer_switch_time_info: extern fn(params: *const Time, double_buffer_index: i32, direct_process: ASIOBool) -> *const Time
		// new callback with time info. makes ASIOGetSamplePosition() and various
		// calls to ASIOGetSampleRate obsolete,
		// and allows for timecode sync etc. to be preferred; will be used if
		// the driver calls asioMessage with selector kAsioSupportsTimeInfo.
}


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
		pub fn get_clock_sources(&self, clocks: *mut ClockSource, num_sources: *mut i32) -> ASIOError;
		pub fn set_clock_source(&self, reference: i32) -> ASIOError;
		pub fn get_sample_position(&self, sample_pos: *mut i64, time_stamp: *mut i64) -> ASIOError;
		pub fn get_channel_info(&self, info: *mut ChannelInfo) -> ASIOError;
		pub fn create_buffers(&self, buffer_infos: *mut BufferInfo, num_channels: i32, buffer_size: i32, callbacks: *const Callbacks) -> ASIOError;
		pub fn dispose_buffers(&self) -> ASIOError;
		pub fn control_panel(&self) -> ASIOError;
		pub fn future(&self, selector: FutureSelector, opt: *mut ()) -> ASIOError;
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
