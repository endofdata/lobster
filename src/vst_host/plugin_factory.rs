use super::IPluginFactory;
use com::AbiTransferable;
use libloading::Library;
use super::pfactory_info::PFactoryInfo;

pub struct PluginFactory {
	vst: Library,
	factory: IPluginFactory
}

impl PluginFactory {
	pub fn load_vst(library_path: &str) -> Result<PluginFactory, Box<dyn std::error::Error>> {
		unsafe {
			let vst = libloading::Library::new(library_path)?;		
			let get_factory: libloading::Symbol<unsafe extern fn() -> IPluginFactory> = vst.get(b"GetPluginFactory")?;
			let factory = get_factory();
			Ok(PluginFactory {
				vst,
				factory: factory
			})
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

impl Drop for PluginFactory {
	fn drop(&mut self) {
		unsafe {
			self.factory.Release();
		}
	}
}