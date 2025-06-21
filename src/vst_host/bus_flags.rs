#[derive(Copy, Clone, Debug)]
#[allow(dead_code)]
pub enum BusFlags
{
	DefaultActive = 1 << 0,
	IsControlVoltage = 1 << 1
}
