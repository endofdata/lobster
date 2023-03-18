#![allow(dead_code)]

mod audio_bus_buffers;
mod bus_direction;
mod bus_flags;
mod bus_info;
mod bus_type;
mod class_cardinality;
mod factory_flags;
mod io_mode;
mod media_type;
mod pclass_info;
mod pfactory_info;
mod vst_event;
mod process_data;
mod process_context;
mod process_setup;
mod routing_info;
mod speaker_arrangement;
mod plugin_factory;
mod plugin_library;
pub mod host;

use std::ffi::c_void;

use com::sys::{GUID, HRESULT};
use com::{interfaces::IUnknown, IID};
use pclass_info::{PClassInfo, PClassInfo2, PClassInfoW};

use self::bus_direction::BusDirection;
use self::bus_info::BusInfo;
use self::io_mode::IoMode;
use self::media_type::MediaType;
use self::pfactory_info::PFactoryInfo;
use self::process_data::ProcessData;
use self::process_setup::ProcessSetup;
use self::routing_info::RoutingInfo;
use self::speaker_arrangement::SpeakerArrangement;
use self::vst_event::Event;

fn utf16_copy(value: &str, target: &mut [u8]) -> usize {
	let mut pos = 0;
	for c in value.encode_utf16().take((target.len() / 2) - 1) {
		target[pos] = (c & 0xFFu16) as u8;
		target[pos + 1] = ((c >> 8) & 0xFFu16) as u8;
		pos += 2;
	}
	target[pos] = 0;
	target[pos + 1] = 0;
	return pos;
}

fn utf8_copy(value: &str, target: &mut [u8]) -> usize {
	let mut pos = 0;
	for c in value.as_bytes().iter().take(target.len() - 1) {
		target[pos] = *c;
		pos += 1;
	}
	target[pos] = 0;
	return pos;
}

fn utf16_copy_w(value: &str, target: &mut [u16]) -> usize {
	let mut pos = 0;
	for c in value.chars().take(target.len() - 1) {
		target[pos] = c as u16;
		pos += 1;
	}
	target[pos] = 0;
	return pos;
}

fn string_from(value: &[u8], is_utf16: bool) -> String {
	if is_utf16 {
		let mut pos = 0;
		let max = value.len();
		let mut conv: Vec<u16> = vec![0; max / 2];
		while pos < max - 1 {
			let codepoint = value[pos] as u16 | ((value[pos + 1] as u16) << 8);
			conv.push(codepoint);
			if codepoint == 0 {
				break;
			}
			pos += 2;
		}
		String::from_utf16(&conv).unwrap()
	} else {
		// TODO: Ist THIS really required?!? Only to get all bytes before the zero and forward it?!?
		String::from_utf8(value.iter().map(|b| *b).take_while(|b| *b != 0u8).collect()).unwrap()
	}
}

#[allow(dead_code)]
fn string_from_w(value: &[u16]) -> String {
	let vec: Vec<u16> = value
		.iter()
		.map(|w| *w)
		.take_while(|w| *w != 0u16)
		.collect();
	String::from_utf16(&vec).unwrap()
}

fn empty_guid() -> com::sys::GUID {
	com::sys::GUID {
		data1: 0,
		data2: 0,
		data3: 0,
		data4: [0; 8],
	}
}

pub fn as_fid_string(guid: &com::sys::GUID) -> String {
	guid.to_string()
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum ErrorSource {
	Other,
	System(HRESULT),
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct Error {
	source: ErrorSource,
	description: String,
}

#[allow(dead_code)]
impl Error {
	pub fn from_other(description: &str) -> Error {
		Error::from_source(description, ErrorSource::Other)
	}

	pub fn from_hresult(description: &str, hresult: HRESULT) -> Error {
		Error::from_source(description, ErrorSource::System(hresult))
	}

	pub fn from_source(description: &str, source: ErrorSource) -> Error {
		Error {
			description: String::from(description),
			source,
		}
	}
}

pub type ParamID = u32;
pub type ParamValue = f64;
pub type TQuarterNotes = f64;
pub type TSamples = i64;
pub type NoteExpressionTypeID = u32;
pub type NoteExpressionValue = f64;

com::interfaces! {

	#[uuid("22888DDB-156E-45AE-8358-B34808190625")]
	pub unsafe interface IPluginBase : IUnknown {
		pub fn initialize(&self, context: *const IUnknown) -> HRESULT;

		pub fn terminate(&self, ) -> HRESULT;
	}


	#[uuid("E831FF31-F2D5-4301-928E-BBEE25697802")]
	pub unsafe interface IComponent : IPluginBase {
		pub fn getControllerClassId(&self, classId: IID) -> HRESULT;

		pub fn setIoMode(&self, mode: IoMode )-> HRESULT;

		pub fn getBusCount(&self, media_type: MediaType, dir: BusDirection) -> i32;

		pub fn getBusInfo(&self, media_type: MediaType, dir: BusDirection, index: i32, bus: *mut BusInfo) -> HRESULT;

		pub fn getRoutingInfo(&self, inInfo: *const RoutingInfo, outInfo: *mut RoutingInfo) -> HRESULT;

		pub fn activateBus(&self, media_type: MediaType, dir: BusDirection, index: i32, state: bool) -> HRESULT;

		pub fn setActive(&self, state: bool) -> HRESULT;

		pub fn setState(&self, state: *const IBStream) -> HRESULT;

		pub fn getState(&self, state: *const IBStream) -> HRESULT;
	}


	#[uuid("58E595CC-DB2D-4969-8B6A-AF8C36A664E5")]
	pub unsafe interface IHostApplication : IUnknown {
		pub fn getName(&self, name: *mut u8) -> i32;

		pub fn createInstance(&self, cid: *const com::IID, iid: *const com::IID, ppv: *mut *mut c_void) -> HRESULT;
	}

	#[uuid("7A4D811C-5211-4A1F-AED9-D2EE0B43BF9F")]
	pub unsafe interface IPluginFactory : IUnknown {
		pub fn getFactoryInfo(&self, factoryInfo: *mut PFactoryInfo) -> HRESULT;

		pub fn countClasses(&self) -> i32;

		pub fn getClassInfo(&self, index: i32, classInfo: *mut PClassInfo) -> HRESULT;

		pub fn createInstance(&self, cidString: *const GUID, iidString: *const GUID, ppv: *mut *mut c_void) -> HRESULT;
	}

	#[uuid("0007B650-F24B-4C0B-A464-EDB9F00B2ABB")]
	pub unsafe interface IPluginFactory2 : IPluginFactory {
		pub fn getClassInfo2(&self, index: i32, classInfo: *mut PClassInfo2) -> HRESULT;
	}

	#[uuid("4555A2AB-C123-4E57-9B12-291036878931")]
	pub unsafe interface IPluginFactory3 : IPluginFactory2 {
		pub fn getClassInfoUnicode(&self, index: i32, classInfo: *mut PClassInfoW) -> HRESULT;

		pub fn setHostContext(&self, context: *mut IUnknown) -> HRESULT;
	}

	#[uuid("C3BF6EA2-3099-4752-9B6B-F9901EE33E9B")]
	pub unsafe interface IBStream: IUnknown {

		pub fn read(&self, buffer: *mut (), numBytes: i32, numBytesRead: *mut i32) -> HRESULT;

		pub fn write(&self, buffer: *const (), numBytes: i32, numBytesWritten: *mut i32) -> HRESULT;

		pub fn seek(&self, pos: i64, mode: i32, result: *mut i64) -> HRESULT;

		pub fn tell(&self, pos: *mut i64) -> HRESULT;
	}

	#[uuid("42043F99-B7DA-453C-A569-E79D9AAEC33D")]
	pub unsafe interface IAudioProcessor : IUnknown	{
		pub fn setBusArrangements(&self, inputs: *const SpeakerArrangement, numIns: i32, outputs: *const SpeakerArrangement, numOuts: i32) -> HRESULT;

		pub fn getBusArrangement (&self, dir: BusDirection, index: i32, arr: *mut SpeakerArrangement) -> HRESULT;

		// TODO: Use enum for symbolicSampleSize
		pub fn canProcessSampleSize(&self, symbolicSampleSize: i32) -> HRESULT;

		pub fn getLatencySamples(&self) -> u32;

		pub fn setupProcessing (&self, setup: *const ProcessSetup) -> HRESULT;

		pub fn setProcessing(&self, state: bool) -> HRESULT;

		pub fn process (&self, data: *const ProcessData) -> HRESULT;

		pub fn getTailSamples(&self) -> u32;
	}

	#[uuid("A4779663-0BB6-4A56-B443-84A8466FEB9D")]
	pub unsafe interface IParameterChanges : IUnknown {
		pub fn getParameterCount(&self) -> i32;

		pub fn getParameterData(&self, index: i32) -> *mut IParamValueQueue;

		pub fn addParameterData(&self, id: *const ParamID, index: *mut i32) -> *mut IParamValueQueue;
	}

	#[uuid("01263A18-ED07-4F6F-98C9-D3564686F9BA")]
	pub unsafe interface IParamValueQueue : IUnknown {
		pub fn getParameterId(&self) -> ParamID;

		pub fn getPointCount(&self) -> i32;

		pub fn getPoint(&self, index: i32, sampleOffset: *mut i32, value: *mut ParamValue) -> HRESULT;

		pub fn addPoint(&self, sampleOffset: i32, value: ParamValue, index: *mut i32) -> HRESULT;
	}


	#[uuid("3A2C4214-3463-49FE-B2C4-F397B9695A44")]
	pub unsafe interface IEventList : IUnknown {
		pub fn getEventCount(&self) -> i32;

		pub fn getEvent(&self, index: i32, e: *mut Event) -> HRESULT;

		pub fn addEvent(&self, e: *const Event) -> HRESULT;
	}


}
