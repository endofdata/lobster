use super::IPluginFactory;
use libloading::Library;
use super::pfactory_info::PFactoryInfo;

pub struct PluginFactory {
	vst: Box<Library>,
	factory: IPluginFactory
}

impl PluginFactory {
	pub fn load_vst(library_path: &str) -> Result<PluginFactory, Box<dyn std::error::Error>> {
		unsafe {
			let vst = libloading::Library::new(library_path)?;		
			let get_factory: libloading::Symbol<unsafe extern fn() -> Option<IPluginFactory>> = vst.get(b"GetPluginFactory")?;
			match get_factory() {
				Some(factory) => {
					Ok(PluginFactory {
						vst: Box::new(vst),
						factory: factory
					})
				},
				// TODO: How to transport this error info in a std::error::Error?
				None => panic!("Failed to call GetPluginFactory()")
			}
		}
	}

	pub fn get_factory_info(&self) -> PFactoryInfo {
		let mut factory_info = PFactoryInfo::new();
		unsafe {
			self.factory.getFactoryInfo(&mut factory_info);
		}
		factory_info
	}
}

