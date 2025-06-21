#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

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

use windows::core::{interface, GUID, HRESULT, IUnknown, IUnknown_Vtbl};
use pclass_info::{PClassInfo, PClassInfo2, PClassInfoW};

use asiolib::ASIOError;

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

fn empty_guid() -> windows::core::GUID {
	windows::core::GUID {
		data1: 0,
		data2: 0,
		data3: 0,
		data4: [0; 8],
	}
}

fn as_fid_string(guid: &windows::core::GUID) -> String {
	// TODO: Check format (was: guid.to_string())
	format!("{:?}", guid)
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum ErrorSource {
	Other,
	System(HRESULT),
	ASIO(ASIOError)
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct Error {
	source: ErrorSource,
	description: String,
}

#[allow(dead_code)]
impl Error {
	fn from_other(description: &str) -> Error {
		Error::from_source(description, ErrorSource::Other)
	}

	fn from_hresult(description: &str, hresult: HRESULT) -> Error {
		Error::from_source(description, ErrorSource::System(hresult))
	}

	fn from_asio(description: &str, asio: ASIOError) -> Error {
		Error::from_source(description, ErrorSource::ASIO(asio))
	}

	fn from_source(description: &str, source: ErrorSource) -> Error {
		Error {
			description: description.into(),
			source,
		}
	}
}

impl From<asiolib::ErrorSource> for ErrorSource {
	fn from(value: asiolib::ErrorSource) -> Self {
		match value {
			asiolib::ErrorSource::ASIO(asio) => ErrorSource::ASIO(asio),
			asiolib::ErrorSource::System(hr) => ErrorSource::System(hr),
			_ => ErrorSource::Other
		}
	}
}

impl From<asiolib::Error> for Error {
	fn from(value: asiolib::Error) -> Self {
		Error {
			description: format!("{:?}", value),
			source: (*value.get_source()).into()
		}
	}
}

pub type ParamID = u32;
pub type ParamValue = f64;
pub type TQuarterNotes = f64;
pub type TSamples = i64;
pub type NoteExpressionTypeID = u32;
pub type NoteExpressionValue = f64;

#[interface("22888DDB-156E-45AE-8358-B34808190625")]
pub unsafe trait IPluginBase : IUnknown {
	fn initialize(&self, context: *const IUnknown) -> HRESULT;

	fn terminate(&self, ) -> HRESULT;
}

#[interface("E831FF31-F2D5-4301-928E-BBEE25697802")]
pub unsafe trait IComponent : IPluginBase {
	fn getControllerClassId(&self, classId: GUID) -> HRESULT;

	fn setIoMode(&self, mode: IoMode )-> HRESULT;

	fn getBusCount(&self, media_type: MediaType, dir: BusDirection) -> i32;

	fn getBusInfo(&self, media_type: MediaType, dir: BusDirection, index: i32, bus: *mut BusInfo) -> HRESULT;

	fn getRoutingInfo(&self, inInfo: *const RoutingInfo, outInfo: *mut RoutingInfo) -> HRESULT;

	fn activateBus(&self, media_type: MediaType, dir: BusDirection, index: i32, state: bool) -> HRESULT;

	fn setActive(&self, state: bool) -> HRESULT;

	fn setState(&self, state: *const IBStream) -> HRESULT;

	fn getState(&self, state: *const IBStream) -> HRESULT;
}


#[interface("58E595CC-DB2D-4969-8B6A-AF8C36A664E5")]
pub unsafe trait IHostApplication : IUnknown {
	fn getName(&self, name: *mut u8) -> i32;

	fn createInstance(&self, cid: *const GUID, iid: *const GUID, ppv: *mut *mut c_void) -> HRESULT;
}

#[interface("7A4D811C-5211-4A1F-AED9-D2EE0B43BF9F")]
pub unsafe trait IPluginFactory : IUnknown {
	fn getFactoryInfo(&self, factoryInfo: *mut PFactoryInfo) -> HRESULT;

	fn countClasses(&self) -> i32;

	fn getClassInfo(&self, index: i32, classInfo: *mut PClassInfo) -> HRESULT;

	fn createInstance(&self, cidString: *const GUID, iidString: *const GUID, ppv: *mut *mut c_void) -> HRESULT;
}

#[interface("0007B650-F24B-4C0B-A464-EDB9F00B2ABB")]
pub unsafe trait IPluginFactory2 : IPluginFactory {
	fn getClassInfo2(&self, index: i32, classInfo: *mut PClassInfo2) -> HRESULT;
}

#[interface("4555A2AB-C123-4E57-9B12-291036878931")]
pub unsafe trait IPluginFactory3 : IPluginFactory2 {
	fn getClassInfoUnicode(&self, index: i32, classInfo: *mut PClassInfoW) -> HRESULT;

	fn setHostContext(&self, context: *mut IUnknown) -> HRESULT;
}

#[interface("C3BF6EA2-3099-4752-9B6B-F9901EE33E9B")]
pub unsafe trait IBStream: IUnknown {

	fn read(&self, buffer: *mut (), numBytes: i32, numBytesRead: *mut i32) -> HRESULT;

	fn write(&self, buffer: *const (), numBytes: i32, numBytesWritten: *mut i32) -> HRESULT;

	fn seek(&self, pos: i64, mode: i32, result: *mut i64) -> HRESULT;

	fn tell(&self, pos: *mut i64) -> HRESULT;
}

#[interface("42043F99-B7DA-453C-A569-E79D9AAEC33D")]
pub unsafe trait IAudioProcessor : IUnknown	{
	fn setBusArrangements(&self, inputs: *const SpeakerArrangement, numIns: i32, outputs: *const SpeakerArrangement, numOuts: i32) -> HRESULT;

	fn getBusArrangement (&self, dir: BusDirection, index: i32, arr: *mut SpeakerArrangement) -> HRESULT;

	// TODO: Use enum for symbolicSampleSize
	fn canProcessSampleSize(&self, symbolicSampleSize: i32) -> HRESULT;

	fn getLatencySamples(&self) -> u32;

	fn setupProcessing (&self, setup: *const ProcessSetup) -> HRESULT;

	fn setProcessing(&self, state: bool) -> HRESULT;

	fn process (&self, data: *const ProcessData) -> HRESULT;

	fn getTailSamples(&self) -> u32;
}

#[interface("A4779663-0BB6-4A56-B443-84A8466FEB9D")]
pub unsafe trait IParameterChanges : IUnknown {
	fn getParameterCount(&self) -> i32;

	fn getParameterData(&self, index: i32) -> *mut IParamValueQueue;

	fn addParameterData(&self, id: *const ParamID, index: *mut i32) -> *mut IParamValueQueue;
}

#[interface("01263A18-ED07-4F6F-98C9-D3564686F9BA")]
pub unsafe trait IParamValueQueue : IUnknown {
	fn getParameterId(&self) -> ParamID;

	fn getPointCount(&self) -> i32;

	fn getPoint(&self, index: i32, sampleOffset: *mut i32, value: *mut ParamValue) -> HRESULT;

	fn addPoint(&self, sampleOffset: i32, value: ParamValue, index: *mut i32) -> HRESULT;
}


#[interface("3A2C4214-3463-49FE-B2C4-F397B9695A44")]
pub unsafe trait IEventList : IUnknown {
	fn getEventCount(&self) -> i32;

	fn getEvent(&self, index: i32, e: *mut Event) -> HRESULT;

	fn addEvent(&self, e: *const Event) -> HRESULT;
}
