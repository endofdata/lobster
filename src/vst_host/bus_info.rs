use super::{media_type::MediaType, bus_direction::BusDirection, bus_type::BusType, bus_flags::BusFlags};

#[repr(C)]
#[allow(dead_code)]
pub struct BusInfo
{
	media_type: MediaType,
	direction: BusDirection,
	channel_count: i32,
	name: String,
	bus_type: BusType,
	flags: BusFlags,
}

