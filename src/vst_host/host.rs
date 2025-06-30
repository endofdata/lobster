use super::{plugin_library::PluginLibrary, Error};
use asiolib::device::Device;
use windows::core::GUID;

pub struct BufferHandler {
}

impl BufferHandler {
	pub fn new() -> BufferHandler {
		BufferHandler {  }
	}

	pub fn process(&mut self, _buffers: &mut [Box<[f64]>], _input_count: usize, _output_count: usize) {
		// TODO: handle audio buffers
	}
}

pub struct Host {
	device: Device,
	buffer_handler: BufferHandler,
	plugins: Vec<PluginLibrary>
}

impl Host {
	pub fn new(clsid: &GUID) -> Result<Host, Error> {
		let mut handler = BufferHandler::new();
		Device::new(clsid, |buffers, in_count, out_count| handler.process(buffers, in_count, out_count))
			.or_else(|e| Err(e.into()))
			.and_then(|device|
				Ok(Host {
					device,
					buffer_handler: handler,
					plugins: Vec::<PluginLibrary>::new()
				}))
	}

	pub fn add_plugin_library(&mut self, path: &str) -> Result<String, Error> {
		// TODO: check whether plugin already loaded
		match PluginLibrary::load(path) {
			Ok(lib) => {
				let id = lib.get_id().to_string();
				self.plugins.push(lib);
				Ok(id)
			},
			Err(err) => Err(err)
		}
	}

	pub fn get_plugin_library(&self, id: &str) -> Result<&PluginLibrary, Error> {
		self.plugins.iter().find(|p| p.get_id() == id)
			.ok_or_else(|| Error::from_other("Invalid VST id"))
	}

	pub fn remove_plugin_library(&mut self, id: &str) -> bool {
		let mut found_match = false;
		// retain all whose id does not match requested id
		self.plugins.retain(|lib| {
			if lib.get_id() == id {
				found_match = true;
				false
			}
			else {
				true
			}
		});
		found_match
	}

	pub fn remove_all_plugin_libraries(&mut self) {
		self.plugins.clear();
	}

	fn process_buffers(input: Vec<Vec<f64>>, outputs: &mut [Vec<f64>]) {
		let ins = input.len() as i32;
		let outs = outputs.len() as i32;

		if ins >= 1 {
			if outs == 2 {
				for o in 0..outs {
					let mut proc = input[0].iter().map(|s| *s * 1.0);
					let dst = outputs[o as usize].iter_mut();
					for target in dst {
						*target = proc.next().expect("Not enough input data");
					}
				}
			}
		}
	}
}

impl Drop for Host {
	fn drop(&mut self) {
		println!("Dropping host");
		self.remove_all_plugin_libraries();
	}
}