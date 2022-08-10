use super::{NoteExpressionValue, TQuarterNotes};


#[derive(Copy, Clone, Debug)]
pub enum NoteIDUserRange
{
	NoteIDUserRangeLowerBound = -10000, 
	NoteIDUserRangeUpperBound = -1000,
}

#[derive(Copy, Clone, Debug)]
pub struct NoteOnEvent
{
	channel: i16,
	pitch: i16,
	tuning: f32,
	velocity: f32,
	length: i32,
	note_id: i32
}

//------------------------------------------------------------------------
/** Note-off event specific data. Used in \ref Event (union)
\ingroup vstEventGrp 
*/
#[derive(Copy, Clone, Debug)]
struct NoteOffEvent
{
	channel: i16,
	pitch: i16,
	velocity: f32,
	note_id: i32,
	tuning: f32,
}

/** Value for DataEvent::type */
#[repr(u32)]
#[derive(Copy, Clone, Debug)]
pub enum DataTypes
{
	MidiSysEx = 0
}

//------------------------------------------------------------------------
/** Data event specific data. Used in \ref Event (union)
\ingroup vstEventGrp 
*/
#[derive(Copy, Clone, Debug)]
struct DataEvent
{
	size: u32,
	data_type: u32,
	bytes: *const u8

}

//------------------------------------------------------------------------
/** PolyPressure event specific data. Used in \ref Event (union)
\ingroup vstEventGrp
*/
#[derive(Copy, Clone, Debug)]
struct PolyPressureEvent
{
	channel: i16,
	pitch: i16,
	pressure: f32,
	note_id: i32,
}

//------------------------------------------------------------------------
/** Chord event specific data. Used in \ref Event (union)
\ingroup vstEventGrp 
*/
#[derive(Copy, Clone, Debug)]
struct ChordEvent
{
	root: i16,
	bass_note: i16,
	mask: i16,
	text_len: u16,

	text: *const char
}

//------------------------------------------------------------------------
/** Scale event specific data. Used in \ref Event (union)
\ingroup vstEventGrp 
*/
#[derive(Copy, Clone, Debug)]
struct ScaleEvent
{
	root: i16,
	mask: i16,
	text_len: u16,

	text: *const char
}


//------------------------------------------------------------------------
/** NoteExpressionTypeIDs describes the type of the note expression.
VST predefines some types like volume, pan, tuning by defining their ranges and curves.
Used by NoteExpressionEvent::typeId and NoteExpressionTypeID::typeId
\see NoteExpressionTypeInfo
*/
#[repr(u32)]
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum NoteExpressionTypeID
{
	VolumeTypeID = 0,
	PanTypeID,
	TuningTypeID,


	VibratoTypeID,
	ExpressionTypeID,
	BrightnessTypeID,
	TextTypeID,
	PhonemeTypeID,

	CustomStart = 100000,
	CustomEnd   = 200000,
	
	InvalidTypeID = 0xFFFFFFFF
}

#[derive(Copy, Clone, Debug)]
pub struct NoteExpressionValueEvent
{
	type_id: NoteExpressionTypeID,
	note_id: i32,
	value: NoteExpressionValue
}

//------------------------------------------------------------------------
/** Note Expression Text event. Used in Event (union)
A Expression event affects one single playing note. \sa INoteExpressionController

\see NoteExpressionTypeInfo
*/
#[derive(Copy, Clone, Debug)]
struct NoteExpressionTextEvent
{
	type_id: NoteExpressionTypeID,
	note_id: i32,
	text_len: u32,
	text: *const char

}

//------------------------------------------------------------------------
/** Legacy MIDI CC Out event specific data. Used in \ref Event (union)
\ingroup vstEventGrp
- [released: 3.6.12]

This ind of event is reserved for generating MIDI CC as output event for Event Bus during the process call.
 */
#[derive(Copy, Clone, Debug)]
struct LegacyMIDICCOutEvent
{
	control_number: u8,
	channel: i8,
	value: i8,
	value2: i8,
}

/** Event Flags - used for Event::flags */
#[repr(u16)]
#[derive(PartialEq, Debug, Eq, Hash, Clone)]
pub enum EventFlags
{
	IsLive = 1 << 0,

	UserReserved1 = 1 << 14,
	UserReserved2 = 1 << 15
}

/**  Event Types - used for Event::type */
#[repr(u16)]
#[derive(PartialEq, Debug, Eq, Hash, Clone)]
pub enum EventTypes
{
	NoteOnEvent       = 0,
	NoteOffEvent      = 1,
	DataEvent         = 2,
	PolyPressureEvent = 3,
	NoteExpressionValueEvent = 4,
	NoteExpressionTextEvent  = 5,
	ChordEvent        = 6,
	ScaleEvent        = 7,
	LegacyMIDICCOutEvent = 65535
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union EventData
{
	note_on: NoteOnEvent,
	note_off: NoteOffEvent,
	data: DataEvent,
	poly_pressure: PolyPressureEvent,
	note_expression_value: NoteExpressionValueEvent,
	note_expression_text: NoteExpressionTextEvent,
	chord: ChordEvent,
	scale: ScaleEvent,
	midi_ccout: LegacyMIDICCOutEvent,
}

//------------------------------------------------------------------------
/** Event 
\ingroup vstEventGrp
Structure representing a single Event of different types associated to a specific event (\ref Event) bus.
*/
#[repr(C)]
pub struct Event
{
	bus_index: i32,
	sample_offset: i32,
	ppq_position: TQuarterNotes,
	flags: u16,

	event_type: EventTypes,
	event_data: EventData
}
