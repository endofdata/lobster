use super::{IPluginFactory,IPluginFactory2,IPluginFactory3, as_fid_string, IComponent};
use super::factory_flags::FactoryFlags;
use com::Interface;
use com::sys::FAILED;
use libloading::Library;
use super::pfactory_info::PFactoryInfo;
use super::pclass_info::{PClassInfo, PClassInfo2, PClassInfoW, ClassInfo};
use super::Error;

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

#[allow(dead_code)]
pub struct PluginFactory {
	factory: IPluginFactory,
	factory_2: Option<IPluginFactory2>,
	factory_3: Option<IPluginFactory3>,
	pub vendor: String,
	pub url: String,
	pub email: String,
	pub flags: FactoryFlags
}

#[allow(dead_code)]
impl PluginFactory {
	fn new(factory: IPluginFactory) -> PluginFactory {
		let mut factory_info = PFactoryInfo::new();
		unsafe {
			factory.getFactoryInfo(&mut factory_info);
		}
		let factory_2 = factory.query_interface::<IPluginFactory2>();
		let factory_3 = factory.query_interface::<IPluginFactory3>();
		
		PluginFactory {
			factory,
			factory_2: factory_2,
			factory_3: factory_3,
			vendor: factory_info.get_vendor(),
			url: factory_info.get_url(),
			email: factory_info.get_email(),
			flags: factory_info.get_flags()
		}
	}

	// TODO: Create an IAudioProcessor instead of IComponent
	pub fn create_audio_module(&self) -> Result<IComponent, Error> {
		self.create_component::<IComponent>("Audio Module Class")
	}

	// TODO: Create an IEditController instead of IComponent
	pub fn create_component_controller(&self) -> Result<IComponent, Error> {
		self.create_component::<IComponent>("Component Controller Class")
	}

	fn count_classes(&self) -> usize {
		unsafe {
			self.factory.countClasses() as usize
		}
	}

	fn get_class_info(&self, index: usize) -> Option<ClassInfo> {
		if index >= self.count_classes() {
			return None
		}
		match &self.factory_3 {
			Some(factory_3) => {
				let mut pclass_info_w = PClassInfoW::new();
				unsafe {
					factory_3.getClassInfoUnicode(index as i32, &mut pclass_info_w);
					Some(ClassInfo::from_class_info_w(&pclass_info_w))
				}
			},
			None => {
				match &self.factory_2 {
					Some(factory_2) => {
						let mut pclass_info_2 = PClassInfo2::new();
						unsafe {
							factory_2.getClassInfo2(index as i32, &mut pclass_info_2);
							Some(ClassInfo::from_class_info_2(&pclass_info_2))
						}
					},
					None => {
						let mut pclass_info = PClassInfo::new();
						unsafe {
							self.factory.getClassInfo(index as i32, &mut pclass_info);
							Some(ClassInfo::from_class_info(&pclass_info))
						}
					}
				}
			}
		}

	}

	fn create_component<T: Interface>(&self, category: &str) -> Result<T, Error> {
		match self.find_class_info(category) {
			Some(class_info) => self.create_instance::<T>(&class_info.cid, &T::IID),
			None => Err(Error::from_other(&format!("No class for category {}.", category)))
		}
	}

	fn find_class_info(&self, category: &str) -> Option<ClassInfo> {
		for c in 0..self.count_classes() {
			if let Some(class_info) = self.get_class_info(c) {
					if class_info.category == category {
						return Some(class_info);
					}
				}
		}
		None
	}

	fn create_instance<T>(&self, cid: &com::sys::CLSID, iid: &com::sys::IID) -> Result<T, Error> {
		let mut opt_instance : Option<T> = None;

		let hr = unsafe {
			// about the beauty of type-safety...
			self.factory.createInstance(cid, iid, &mut opt_instance as *mut _ as *mut *mut std::ffi::c_void)
		};

		if FAILED(hr) {
			return Err(Error::from_hresult("Cannot create instance", hr));
		}

		match opt_instance {
			Some(instance) => Ok(instance),
			None => Err(Error::from_other("Create instance returned null"))
		}
	}
}

