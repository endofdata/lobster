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
mod plugin_library;
mod plugin;
pub mod host;
pub mod error;

pub use error::*;

use std::ffi::c_void;

use windows::core::{interface, GUID, HRESULT, IUnknown, IUnknown_Vtbl};
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
	match target.len() {
		0 => 0,
		1 => {
			target[0] = 0;
			0
		},
		len => {
			let mut bndry = usize::min(len - 1, value.len());
			while bndry > 0 && !value.is_char_boundary(bndry) {
				bndry -= 1;
			}
			if bndry > 0 {
				for (pos, c) in value.bytes().take(bndry).enumerate() {
					target[pos] = c;
				}
				target[bndry] = 0;
				bndry
			}
			else {
				0
			}
		}
	}
}

fn utf16_copy_w(value: &str, target: &mut [u16]) -> usize {
	let mut pos = 0;
	for c in value.encode_utf16().take(target.len() - 1) {
		target[pos] = c;
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

pub fn as_fid_string(guid: &windows::core::GUID) -> String {
	// TODO: Check format (was: guid.to_string())
	format!("{:?}", guid)
}

pub type ParamID = u32;
pub type ParamValue = f64;
pub type TQuarterNotes = f64;
pub type TSamples = i64;
pub type NoteExpressionTypeID = u32;
pub type NoteExpressionValue = f64;
pub type String128 = *mut [char;128];
// TODO: Check VST FIDString format for separators or brackets
pub type FIDString = *mut [u8;32];
pub type UnitID = i32;
pub type wchar_t = u32;

pub const VST_AUDIO_EFFECT_CLASS : &'static str = "Audio Module Class";

//------------------------------------------------------------------------
/** Flags used for IComponentHandler::restartComponent */
pub enum RestartFlags
{
	/// The Component should be reloaded
	/// The host has to unload completely the plug-in (controller/processor) and reload it.
	/// \[SDK 3.0.0\]
	kReloadComponent			= 1 << 0,

	/// Input / Output Bus configuration has changed
	/// The plug-in informs the host that either the bus configuration or the bus count has changed.
	/// The host has to deactivate the plug-in, asks the plug-in for its wanted new bus configurations,
	/// adapts its processing graph and reactivate the plug-in.
	/// \[SDK 3.0.0\]
	kIoChanged					= 1 << 1,

	/// Multiple parameter values have changed  (as result of a program change for example)
	/// The host invalidates all caches of parameter values and asks the edit controller for the current values.
	/// \[SDK 3.0.0\]
	kParamValuesChanged			= 1 << 2,

	/// Latency has changed
	/// The plug informs the host that its latency has changed, getLatencySamples should return the new latency after setActive (true) was called
	/// The host has to deactivate and reactivate the plug-in, then afterwards the host could ask for the current latency (getLatencySamples)
	/// see IAudioProcessor::getLatencySamples
	/// \[SDK 3.0.0\]
	kLatencyChanged				= 1 << 3,

	/// Parameter titles, default values or flags (ParameterFlags) have changed
	/// The host invalidates all caches of parameter infos and asks the edit controller for the current infos.
	/// \[SDK 3.0.0\]
	kParamTitlesChanged			= 1 << 4,

	/// MIDI Controllers and/or Program Changes Assignments have changed
	/// The plug-in informs the host that its MIDI-CC mapping has changed (for example after a MIDI learn or new loaded preset)
	/// or if the stepCount or UnitID of a ProgramChange parameter has changed.
	/// The host has to rebuild the MIDI-CC => parameter mapping (getMidiControllerAssignment)
	/// and reread program changes parameters (stepCount and associated unitID)
	/// \[SDK 3.0.1\]
	kMidiCCAssignmentChanged	= 1 << 5,

	/// Note Expression has changed (info, count, PhysicalUIMapping, ...)
	/// Either the note expression type info, the count of note expressions or the physical UI mapping has changed.
	/// The host invalidates all caches of note expression infos and asks the edit controller for the current ones.
	/// See INoteExpressionController, NoteExpressionTypeInfo and INoteExpressionPhysicalUIMapping
	/// \[SDK 3.5.0\]
	kNoteExpressionChanged		= 1 << 6,

	/// Input / Output bus titles have changed
	/// The host invalidates all caches of bus titles and asks the edit controller for the current titles.
	/// \[SDK 3.5.0\]
	kIoTitlesChanged			= 1 << 7,

	/// Prefetch support has changed
	/// The plug-in informs the host that its PrefetchSupport has changed
	/// The host has to deactivate the plug-in, calls IPrefetchableSupport::getPrefetchableSupport and reactivate the plug-in
	/// see IPrefetchableSupport
	/// \[SDK 3.6.1\]
	kPrefetchableSupportChanged = 1 << 8,

	/// RoutingInfo has changed
	/// The plug-in informs the host that its internal routing (relation of an event-input-channel to an audio-output-bus) has changed
	/// The host ask the plug-in for the new routing with IComponent::getRoutingInfo, \ref vst3Routing
	/// see IComponent
	/// \[SDK 3.6.6\]
	kRoutingInfoChanged			= 1 << 9,

	/// Key switches has changed (info, count)
	/// Either the Key switches info, the count of Key switches has changed.
	/// The host invalidates all caches of Key switches infos and asks the edit controller (IKeyswitchController) for the current ones.
	/// See IKeyswitchController
	/// \[SDK 3.7.3\]
	kKeyswitchChanged			= 1 << 10
}

pub enum ParameterFlags
{
	/// no flags wanted
	kNoFlags		 = 0,
	/// parameter can be automated
	kCanAutomate	 = 1 << 0,
	/// parameter cannot be changed from outside the plug-in (implies that kCanAutomate is NOT set)
	kIsReadOnly		 = 1 << 1,
	/// attempts to set the parameter value out of the limits will result in a wrap around \[SDK 3.0.2\]
	kIsWrapAround	 = 1 << 2,
	/// parameter should be displayed as list in generic editor or automation editing \[SDK 3.1.0\]
	kIsList			 = 1 << 3,
	/// parameter should be NOT displayed and cannot be changed from outside the plug-in
	kIsHidden		 = 1 << 4,
	/// (implies that kCanAutomate is NOT set and kIsReadOnly is set) \[SDK 3.7.0\]


	/// parameter is a program change (unitId gives info about associated unit
	kIsProgramChange = 1 << 15,
	/// - see ef vst3ProgramLists)

	/// special bypass parameter (only one allowed): plug-in can handle bypass
	/// (highly recommended to export a bypass parameter for effect plug-in)
	kIsBypass		 = 1 << 16

}

pub struct ParameterInfo
{
	/// unique identifier of this parameter (named tag too)
	id: ParamID,
	/// parameter title (e.g. "Volume")
	title: String128,
	/// parameter shortTitle (e.g. "Vol")
	shortTitle: String128,
	/// parameter unit (e.g. "dB")
	units: String128,
	/// number of discrete steps (0: continuous, 1: toggle, discrete value otherwise
	stepCount: i32,
	/// (corresponding to max - min, for example: 127 for a min = 0 and a max = 127) - see ef vst3ParameterIntro)

	/// default normalized value [0,1] (in case of discrete value: defaultNormalizedValue = defDiscreteValue / stepCount)
	defaultNormalizedValue: ParamValue,
	/// id of unit this parameter belongs to (see ef vst3Units)
	unitId: UnitID,
	/// ParameterFlags (see below)
	flags: i32,
}


#[interface("22888DDB-156E-45AE-8358-B34808190625")]
pub unsafe trait IPluginBase : IUnknown {
	pub fn initialize(&self, context: *const IUnknown) -> HRESULT;

	pub fn terminate(&self, ) -> HRESULT;
}

#[interface("E831FF31-F2D5-4301-928E-BBEE25697802")]
pub unsafe trait IComponent : IPluginBase {
	pub fn getControllerClassId(&self, classId: *mut GUID) -> HRESULT;

	pub fn setIoMode(&self, mode: IoMode )-> HRESULT;

	pub fn getBusCount(&self, media_type: MediaType, dir: BusDirection) -> i32;

	pub fn getBusInfo(&self, media_type: MediaType, dir: BusDirection, index: i32, bus: *mut BusInfo) -> HRESULT;

	pub fn getRoutingInfo(&self, inInfo: *const RoutingInfo, outInfo: *mut RoutingInfo) -> HRESULT;

	pub fn activateBus(&self, media_type: MediaType, dir: BusDirection, index: i32, state: bool) -> HRESULT;

	pub fn setActive(&self, state: bool) -> HRESULT;

	pub fn setState(&self, state: *const IBStream) -> HRESULT;

	pub fn getState(&self, state: *const IBStream) -> HRESULT;
}


#[interface("58E595CC-DB2D-4969-8B6A-AF8C36A664E5")]
pub unsafe trait IHostApplication : IUnknown {
	pub fn getName(&self, name: *mut u8) -> i32;

	pub fn createInstance(&self, cid: *const GUID, iid: *const GUID, ppv: *mut *mut c_void) -> HRESULT;
}

#[interface("7A4D811C-5211-4A1F-AED9-D2EE0B43BF9F")]
pub unsafe trait IPluginFactory : IUnknown {
	pub fn getFactoryInfo(&self, factoryInfo: *mut PFactoryInfo) -> HRESULT;

	pub fn countClasses(&self) -> i32;

	pub fn getClassInfo(&self, index: i32, classInfo: *mut PClassInfo) -> HRESULT;

	pub fn createInstance(&self, cidString: *const GUID, iidString: *const GUID, ppv: *mut *mut c_void) -> HRESULT;
}

#[interface("0007B650-F24B-4C0B-A464-EDB9F00B2ABB")]
pub unsafe trait IPluginFactory2 : IPluginFactory {
	pub fn getClassInfo2(&self, index: i32, classInfo: *mut PClassInfo2) -> HRESULT;
}

#[interface("4555A2AB-C123-4E57-9B12-291036878931")]
pub unsafe trait IPluginFactory3 : IPluginFactory2 {
	pub fn getClassInfoUnicode(&self, index: i32, classInfo: *mut PClassInfoW) -> HRESULT;

	pub fn setHostContext(&self, context: *mut IUnknown) -> HRESULT;
}

#[interface("C3BF6EA2-3099-4752-9B6B-F9901EE33E9B")]
pub unsafe trait IBStream: IUnknown {

	pub fn read(&self, buffer: *mut (), numBytes: i32, numBytesRead: *mut i32) -> HRESULT;

	pub fn write(&self, buffer: *const (), numBytes: i32, numBytesWritten: *mut i32) -> HRESULT;

	pub fn seek(&self, pos: i64, mode: i32, result: *mut i64) -> HRESULT;

	pub fn tell(&self, pos: *mut i64) -> HRESULT;
}

#[interface("42043F99-B7DA-453C-A569-E79D9AAEC33D")]
pub unsafe trait IAudioProcessor : IUnknown	{
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

#[interface("A4779663-0BB6-4A56-B443-84A8466FEB9D")]
pub unsafe trait IParameterChanges : IUnknown {
	pub fn getParameterCount(&self) -> i32;

	pub fn getParameterData(&self, index: i32) -> *mut IParamValueQueue;

	pub fn addParameterData(&self, id: *const ParamID, index: *mut i32) -> *mut IParamValueQueue;
}

#[interface("01263A18-ED07-4F6F-98C9-D3564686F9BA")]
pub unsafe trait IParamValueQueue : IUnknown {
	pub fn getParameterId(&self) -> ParamID;

	pub fn getPointCount(&self) -> i32;

	pub fn getPoint(&self, index: i32, sampleOffset: *mut i32, value: *mut ParamValue) -> HRESULT;

	pub fn addPoint(&self, sampleOffset: i32, value: ParamValue, index: *mut i32) -> HRESULT;
}


#[interface("3A2C4214-3463-49FE-B2C4-F397B9695A44")]
pub unsafe trait IEventList : IUnknown {
	pub fn getEventCount(&self) -> i32;

	pub fn getEvent(&self, index: i32, e: *mut Event) -> HRESULT;

	pub fn addEvent(&self, e: *const Event) -> HRESULT;
}


/// Host callback interface for an edit controller
/// - \[host imp\]
/// - \[released: 3.0.0\]
/// - \[mandatory\]
///
/// Allow transfer of parameter editing to component (processor) via host and support automation.
/// Cause the host to react on configuration changes (restartComponent).
/// see [IEditController]
#[interface("93A0BEA3-0BD0-45DB-8E89-0B0CC1E46AC6")]
unsafe trait IComponentHandler : IUnknown
{
	/// To be called before calling a performEdit (e.g. on mouse-click-down event).
	/// This must be called in the UI-Thread context!
	pub fn beginEdit (&self, id: ParamID) -> HRESULT;

	/// Called between beginEdit and endEdit to inform the handler that a given parameter has a new
	/// value. This must be called in the UI-Thread context!
	pub fn performEdit (&self, id: ParamID, valueNormalized: ParamValue) -> HRESULT;

	/// To be called after calling a performEdit (e.g. on mouse-click-up event).
	/// This must be called in the UI-Thread context!
	pub fn endEdit (&self, id: ParamID) -> HRESULT;

	/// Instructs host to restart the component. This must be called in the UI-Thread context!
	/// [flags] is a combination of [RestartFlags]
	pub fn restartComponent (&self, flags: i32) -> HRESULT;
}

/// Graphical rectangle structure. Used with IPlugView.
#[repr(C)]
pub struct ViewRect
{
	left: i32,
	top: i32,
	right: i32,
	bottom: i32,
}

impl ViewRect {
	pub fn new(left: i32, top: i32, right: i32, bottom: i32) -> ViewRect {
		ViewRect { left, top, right, bottom }
	}

	pub fn get_width(&self) -> i32 {
		self.right - self.left
	}

	pub fn get_height(&self) -> i32 {
		self.bottom - self.top
	}
}

/// Callback interface passed to IPlugView.
/// - \[host imp\]
/// - \[released: 3.0.0\]
/// - \[mandatory\]
///
/// Enables a plug-in to resize the view and cause the host to resize the window.
#[interface("367FAF01-AFA9-4693-8D4D-A2A0ED0882A3")]
unsafe trait IPlugFrame : IUnknown
{
	/// Called to inform the host about the resize of a given view.
	///
	/// Afterwards the host has to call IPlugView::onSize ().
	pub fn resizeView (&self, view: *const IPlugView, newSize: *const ViewRect) -> HRESULT;
}

/// Plug-in definition of a view.
/// - \[plug imp\]
/// - \[released: 3.0.0\]
/// Sizing of a view
///
/// Usually, the size of a plug-in view is fixed. But both the host and the plug-in can cause
/// a view to be resized:
///
/// - Host: If IPlugView::canResize () returns kResultTrue the host will set up the window
/// so that the user can resize it. While the user resizes the window,
/// IPlugView::checkSizeConstraint () is called, allowing the plug-in to change the size to a valid
/// a valid supported rectangle size. The host then resizes the window to this rect and has to call IPlugView::onSize ().
///
/// - Plug-in: The plug-in can call IPlugFrame::resizeView () and cause the host to resize the window
///
/// Afterwards, in the same callstack, the host has to call IPlugView::onSize () if a resize is needed (size was changed).
/// Note that if the host calls IPlugView::getSize () before calling IPlugView::onSize () (if needed),
/// it will get the current (old) size not the wanted one!
///
/// Here the calling sequence:
/// - plug-in->host: IPlugFrame::resizeView (newSize)
/// - host->plug-in (optional): IPlugView::getSize () returns the currentSize (not the newSize!)
/// - host->plug-in: if newSize is different from the current size: IPlugView::onSize (newSize)
/// - host->plug-in (optional): IPlugView::getSize () returns the newSize
///
/// **Please only resize the platform representation of the view when IPlugView::onSize () is called.**
///
/// Keyboard handling
///
/// The plug-in view receives keyboard events from the host. A view implementation must not handle
/// keyboard events by the means of platform callbacks, but let the host pass them to the view. The host
/// depends on a proper return value when IPlugView::onKeyDown is called, otherwise the plug-in view may
/// cause a malfunction of the host's key command handling.
///
/// see [IPlugFrame], platformUIType
#[interface("5BC32507-D060-49EA-A615-1B522B755B29")]
unsafe trait IPlugView : IUnknown
{
	/// Is Platform UI Type supported
	/// [uiType] : IDString of [platformUIType]
	pub fn isPlatformTypeSupported(&self, uiType: FIDString) -> HRESULT;

	/// The parent window of the view has been created, the (platform) representation of the view
	/// should now be created as well.
	/// Note that the parent is owned by the caller and you are not allowed to alter it in any way
	/// other than adding your own views.
	/// Note that in this call the plug-in could call a IPlugFrame::resizeView ()!
	/// \param parent : platform handle of the parent window or view
	/// \param type : \ref platformUIType which should be created
	pub fn attached(&self, parent: *const std::ffi::c_void, uiType: FIDString) -> HRESULT;

	/// The parent window of the view is about to be destroyed.
	/// You have to remove all your own views from the parent window or view.
	pub fn removed(&self) -> HRESULT;

	/// Handling of mouse wheel.
	pub fn onWheel(&self, distance: f32) -> HRESULT;

	/// Handling of keyboard events : Key Down.
	/// key : unicode code of key
	/// keyCode : virtual keycode for non ascii keys - see \ref VirtualKeyCodes in keycodes.h
	/// modifiers : any combination of modifiers - see \ref KeyModifier in keycodes.h
	/// returns : kResultTrue if the key is handled, otherwise kResultFalse. \n
	/// <b> Please note that kResultTrue must only be returned if the key has really been
	/// handled. </b> Otherwise key command handling of the host might be blocked!
	pub fn onKeyDown(&self, key: wchar_t, keyCode: i16, modifiers: i16) -> HRESULT;

	/// Handling of keyboard events : Key Up.
	/// key : unicode code of key
	/// \param keyCode : virtual keycode for non ascii keys - see \ref VirtualKeyCodes in keycodes.h
	/// \param modifiers : any combination of KeyModifier - see \ref KeyModifier in keycodes.h
	/// \return kResultTrue if the key is handled, otherwise return kResultFalse.
	pub fn onKeyUp(&self, key: wchar_t, keyCode: i16, modifiers: i16) -> HRESULT;

	/// Returns the size of the platform representation of the view.
	pub fn getSize(&self, size: *mut ViewRect) -> HRESULT;

	/// Resizes the platform representation of the view to the given rect. Note that if the plug-in
	/// requests a resize (IPlugFrame::resizeView ()) onSize has to be called afterward.
	pub fn onSize(&self, newSize: *const ViewRect) -> HRESULT;

	/// Focus changed message.
	pub fn onFocus(&self, state: bool) -> HRESULT;

	/// Sets IPlugFrame object to allow the plug-in to inform the host about resizing.
	pub fn setFrame(&self, frame: *const IPlugFrame) -> HRESULT;

	/// Is view sizable by user.
	pub fn canResize(&self) -> HRESULT;

	/// On live resize this is called to check if the view can be resized to the given rect, if not
	/// adjust the rect to the allowed size.
	pub fn checkSizeConstraint(&self, rect: *mut ViewRect) -> HRESULT;
}


#[interface("DCD7BBE3-7742-448D-A874-AACC979C759E")]
pub unsafe trait IEditController : IPluginBase {
	pub fn setComponentState(&self, state: *const IBStream) -> HRESULT;

	/** Sets the controller state. */
	pub fn setState(&self, state: *const IBStream) -> HRESULT;

	/** Gets the controller state. */
	pub fn getState(&self, state: *const IBStream) -> HRESULT;

	// parameters -------------------------
	/** Returns the number of parameters exported. */
	pub fn getParameterCount(&self) -> i32;
	/** Gets for a given index the parameter information. */
	pub fn getParameterInfo(&self, paramIndex: i32, info: *mut ParameterInfo  /*out*/) -> HRESULT;

	/** Gets for a given paramID and normalized value its associated string representation. */
	pub fn getParamStringByValue(&self, id: ParamID, valueNormalized: ParamValue /*in*/, paramStringOut: String128) -> HRESULT;
	/** Gets for a given paramID and string its normalized value. */
	pub fn getParamValueByString(&self, id: ParamID, value: *const char, valueNormalized: *mut ParamValue) -> HRESULT;

	/** Returns for a given paramID and a normalized value its plain representation
		(for example -6 for -6dB - see ef vst3AutomationIntro). */
	pub fn normalizedParamToPlain(&self, id: ParamID, valueNormalized: ParamValue) -> ParamValue;
	/** Returns for a given paramID and a plain value its normalized value. (see ef vst3AutomationIntro) */
	pub fn plainParamToNormalized(&self, id: ParamID, plainValue: ParamValue) -> ParamValue;

	/** Returns the normalized value of the parameter associated to the paramID. */
	pub fn getParamNormalized(&self, id: ParamID) -> ParamValue;
	/** Sets the normalized value to the parameter associated to the paramID. The controller must never
	    pass this value-change back to the host via the IComponentHandler. It should update the according
		GUI element(s) only!*/
	pub fn setParamNormalized(&self, id: ParamID, value: ParamValue) -> HRESULT;

	// handler ----------------------------
	/** Gets from host a handler which allows the Plugin-in to communicate with the host.
		Note: This is mandatory if the host is using the IEditController! */
	pub fn setComponentHandler(&self, handler: *const IComponentHandler) -> HRESULT;

	// view -------------------------------
	/** Creates the editor view of the plug-in, currently only "editor" is supported, see ef ViewType.
		The life time of the editor view will never exceed the life time of this controller instance. */
	pub fn createView(&self, name: FIDString) -> *const IPlugView;

}

#[cfg(test)]
mod test {
	use super::{utf8_copy, utf16_copy};

	#[test]
	pub fn can_copy_utf16() {
		let value = "Thornton Wilder";
		let mut target = [0u8; 32];
		let byte_count = utf16_copy(value, &mut target);

		assert_eq!(byte_count & 1, 0, "byte count should be an even number");

		let utf_16 : &[u16] = unsafe { std::slice::from_raw_parts((&target as *const u8) as *const u16, byte_count / 2) };
		let result = String::from_utf16_lossy(utf_16);

		assert_eq!(result, value, "copying as utf16 should be lossless");

		let mut odd_target = [0u8; 7];
		let byte_count = utf16_copy("123", &mut odd_target);

		assert_eq!(byte_count, 4, "utf16_copy should use only even number of target bytes");
	}

	#[test]
	pub fn can_copy_utf8() {
		let mut target = [0u8; 16];

		for (value, expect) in [
			("Simple Text", "Simple Text"),
			// this is German for 'Beautiful shit'
			("Schöne Scheiße", "Schöne Scheiß"),
			// this is German for 'Hangs at the end'
			("Hängt am Ende drüber", "Hängt am Ende "),
			// this is Tulu for 'What do we have here?'
			("ನಮಕ್ ಮುಲ್ಪ ದಾದ ಉಂಡು?", "ನಮಕ್ "),
			("", "")] {
			let copied = utf8_copy(value, &mut target);
			let result = String::from_utf8_lossy(&target[0..copied]);

			assert_eq!(result, expect);
		}
	}
}