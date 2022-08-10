use super::{TSamples, TQuarterNotes};

pub enum StatesAndFlags
{
	Playing          = 1 << 1,
	CycleActive      = 1 << 2,
	Recording        = 1 << 3,

	SystemTimeValid  = 1 << 8,
	ContTimeValid    = 1 << 17,

	ProjectTimeMusicValid = 1 << 9,
	BarPositionValid = 1 << 11,
	CycleValid       = 1 << 12,

	TempoValid       = 1 << 10,
	TimeSigValid     = 1 << 13,
	ChordValid       = 1 << 18,

	SmpteValid       = 1 << 14,
	ClockValid       = 1 << 15
}

#[repr(u16)]
#[derive(PartialEq, Debug, Eq, Hash, Clone)]
pub enum Masks {
	ChordMask = 0x0FFFu16,
	ReservedMask = 0xF000u16
}

pub struct Chord
{
//------------------------------------------------------------------------
	key_note: u8,
	root_note: u8,

	/** Bitmask of a chord. \n
	    1st bit set: minor second; 2nd bit set: major second, and so on. \n
		There is \b no bit for the keynote (root of the chord) because it is inherently always present. \n
		Examples:
		- XXXX 0000 0100 1000 (= 0x0048) -> major chord
		- XXXX 0000 0100 0100 (= 0x0044) -> minor chord
		- XXXX 0010 0100 0100 (= 0x0244) -> minor chord with minor seventh */
	chord_mask: Masks
}

#[repr(u32)]
#[derive(PartialEq, Debug, Eq, Hash, Clone)]
pub enum FrameRateFlags
{
	PullDownRate = 1 << 0,
	DropRate     = 1 << 1
}

pub struct FrameRate
{
	frames_per_second: u32,
	flags: FrameRateFlags
}

pub struct ProcessContext
{
	state: u32,

	sample_rate: f64,
	project_time_samples: TSamples,

	system_time: i64,
	continous_time_samples: TSamples,

	project_time_music: TQuarterNotes,
	bar_position_music: TQuarterNotes,
	cycle_start_music: TQuarterNotes,
	cycle_end_music: TQuarterNotes,

	tempo: f64,
	time_sig_numerator: i32,
	time_sig_denominator: i32,

	chord: Chord,

	smpte_offset_subframes: i32,
	frame_rate: FrameRate,

	samples_to_next_clock: i32,
}
