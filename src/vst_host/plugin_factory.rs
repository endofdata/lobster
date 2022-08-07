use super::IPluginFactory;
use super::factory_flags::FactoryFlags;
use libloading::Library;
use super::pfactory_info::PFactoryInfo;
use super::Error;

pub struct PluginLibrary {
	vst: Library
}

impl PluginLibrary {
	pub fn load(library_path: &str) -> Result<PluginLibrary, Error> {
		unsafe {
			match libloading::Library::new(library_path) {
				Ok(vst) => Ok(PluginLibrary { vst }),
				Err(error) => Err(Error::from_other(&format!("Failed to load VST '{}': {:?}", library_path, error))) 
			}
		}
	}

	pub fn get_factory(&self)  -> Result<PluginFactory, Error> {
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
}

pub struct PluginFactory {
	factory: IPluginFactory,

	pub vendor: String,
	pub url: String,
	pub email: String,
	pub flags: FactoryFlags
}

impl PluginFactory {
	fn new(factory: IPluginFactory) -> PluginFactory {
		let mut factory_info = PFactoryInfo::new();
		unsafe {
			factory.getFactoryInfo(&mut factory_info);
		}
		PluginFactory {
			factory,
			vendor: factory_info.get_vendor(),
			url: factory_info.get_url(),
			email: factory_info.get_email(),
			flags: factory_info.get_flags()
		}
	}

	pub fn count_classes(&self) -> usize {
		unsafe {
			self.factory.countClasses() as usize
		}
	}
}

