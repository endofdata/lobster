mod class_cardinality;
mod factory_flags;
mod pfactory_info;
mod pclass_info;
pub mod plugin_factory;

use com::sys::HRESULT;
use com::interfaces::IUnknown;
use pclass_info::{PClassInfo, PClassInfo2, PClassInfoW};

use self::pfactory_info::PFactoryInfo;

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
		let mut conv : Vec<u16> = vec![0; max / 2];
		while pos < max - 1 {
			let codepoint = value[pos] as u16 | ((value[pos + 1] as u16) << 8);
			conv.push(codepoint);
			if codepoint == 0 {
				break;
			}
			pos += 2;
		}
		String::from_utf16(&conv).unwrap()
	}
	else {
		// TODO: Ist THIS really required?!? Only to get all bytes before the zero and forward it?!?
		String::from_utf8(value.iter().map(|b| *b).take_while(|b| *b != 0u8).collect()).unwrap()
	}
}

#[allow(dead_code)]
fn string_from_w(value: &[u16]) -> String {
	let vec : Vec<u16> = value.iter().map(|w| *w).take_while(|w| *w != 0u16).collect();
	String::from_utf16(&vec).unwrap()
}

fn empty_guid() -> com::sys::GUID {
	com::sys::GUID {
		data1: 0,
		data2: 0,
		data3: 0,
		data4: [0; 8]
	}
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum ErrorSource {
	Other,
	System(HRESULT)
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct Error {
	source: ErrorSource,
	description: String
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
			source 
		}
	}
}


com::interfaces! {

	#[uuid("58E595CC-DB2D-4969-8B6A-AF8C36A664E5")]
	pub unsafe interface IHostApplication : IUnknown {	
		pub fn getName(&self, name: *mut u8) -> i32;

		pub fn createInstance(&self, cid: *const com::IID, iid: *const com::IID, ppv: *mut *mut ()) -> HRESULT;
	}

	#[uuid("7A4D811C-5211-4A1F-AED9-D2EE0B43BF9F")]
	pub unsafe interface IPluginFactory : IUnknown {
		pub fn getFactoryInfo(&self, factoryInfo: *mut PFactoryInfo) -> HRESULT;
		
		pub fn countClasses(&self) -> i32;

		pub fn getClassInfo(&self, index: i32, classInfo: *mut PClassInfo) -> HRESULT;

		pub fn createInstance(&self, cidString: *const u8, iidString: *const u8, ppv: *mut *mut ()) -> HRESULT;
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


}
