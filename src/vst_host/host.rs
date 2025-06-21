use crate::asio_core::asio_device::ASIODeviceType;
use sha256::digest;
use std::collections::hash_map::HashMap;
use super::{plugin_library::PluginLibrary, Error, IAudioProcessor, IComponent};
use crate::asio_core::device_factory::DeviceFactory;

pub struct Host<'a> {
	device: &'a mut dyn ASIODeviceType,
	plugins: HashMap<String, PluginLibrary>
}

impl<'a> Host<'a> {
	pub fn new(clsid: com::CLSID) -> Result<Host<'a>, Error> {
		DeviceFactory::create_device(clsid, Host::process_buffers)
			.or_else(|e| Err(e.into()))
			.and_then(|device|
				Ok(Host {
					device,
					plugins: HashMap::<String, PluginLibrary>::new()
				}))
	}

	pub fn add_plugin(&mut self, path: &str) -> Result<String, Error> {
		let id = digest(path);
		// TODO: check whether plugin already loaded
		match PluginLibrary::load(path) {
			Ok(lib) => {
				let result = id.clone();
				self.plugins.insert(id, lib);
				Ok(result)
			},
			Err(err) => Err(err)
		}
	}

	pub fn get_audio_processor(&self, id: &str) -> Result<IAudioProcessor, Error> {
		self.plugins.get(id)
			.ok_or_else(|| Error::from_other("Invalid VST id"))
			.and_then(|vst| vst.get_audio_processor())
	}

	pub fn get_component(&self, id: &str) -> Result<IComponent, Error> {
		self.plugins.get(id)
			.ok_or_else(|| Error::from_other("Invalid VST id"))
			.and_then(|vst| vst.get_component())
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

	pub fn remove_plugin(&mut self, id: &str) -> bool {
		if let Some(lib) = self.plugins.remove(id) {
			drop(lib);
			return true;
		}
		false
	}

	pub fn remove_all_plugins(&mut self) -> Result<(), Error> {
		let keys: Vec<String> = self.plugins.keys().map(|k| k.clone()).collect();
		for key in keys {
			if let Some(lib) = self.plugins.remove(&key) {
				lib.close()?;
			}
		}
		self.plugins.clear();
		Ok(())
	}

					// device.set_sample_rate(48000.0f64);

				// println!("ASIO device starting");
				// device.start();
				// println!("ASIO Device started");

				// thread::sleep(Duration::from_secs(2));

				// println!("ASIO device stopping");
				// device.stop();
				// println!("ASIO device stopped");

				// asio_core::device_factory::DeviceFactory::drop_device();
}

impl<'a> Drop for Host<'a> {
	fn drop(&mut self) {
		println!("Dropping host");
		self.remove_all_plugins().unwrap();
		DeviceFactory::drop_device();
	}
}