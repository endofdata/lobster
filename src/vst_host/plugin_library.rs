use libloading::Library;
use super::{IPluginFactory, IAudioProcessor};
use super::Error;
use super::plugin_factory::PluginFactory;

pub struct PluginLibrary {
	vst: Library
}

impl PluginLibrary {
	pub fn load(library_path: &str) -> Result<PluginLibrary, Error> {
		unsafe {
			match libloading::Library::new(library_path) {
				Ok(vst) => Ok(PluginLibrary::new(vst)),
				Err(error) => Err(Error::from_other(&format!("Failed to load VST '{}': {:?}", library_path, error))) 
			}
		}
	}

	pub fn new(vst: Library) -> PluginLibrary {
		unsafe {
			let opt_method : Result<libloading::Symbol<unsafe extern fn()>, libloading::Error> = vst.get(b"InitDll");
			match opt_method {
				Ok(init_dll) => init_dll(),
				// method is optional
				Err(_) => ()
			}
		}
		PluginLibrary { vst }
	}

	fn get_factory(&self)  -> Result<PluginFactory, Error> {
		unsafe {
			let opt_method : Result<libloading::Symbol<unsafe extern fn() -> Option<IPluginFactory>>, libloading::Error> = self.vst.get(b"GetPluginFactory");
			match opt_method {			
				Ok(get_factory) => match get_factory() {
					Some(factory) => {
						let _ = factory.countClasses();
						Ok(PluginFactory::new(factory))
					},
					None => Err(Error::from_other("VST.GetPluginFactory() returned null."))
				},
				Err(error) => Err(Error::from_other(&format!("Missing entry point 'GetPluginFactory' in VST: {:?}", error)))
			}
		}
	}

	pub fn get_audio_processor(self) -> Result<IAudioProcessor, Error> {
		match self.get_factory() {
			Ok(factory) => factory.create_audio_processor(),
			Err(error) => Err(error)	
		}
	}
}

impl Drop for PluginLibrary {
	fn drop(&mut self) {
		unsafe {
			let opt_method : Result<libloading::Symbol<unsafe extern fn()>, libloading::Error> = self.vst.get(b"ExitDll");
			match opt_method {
				Ok(exit_dll) => exit_dll(),
				// method is optional
				Err(_) => ()
			}
		}		
	}
}
