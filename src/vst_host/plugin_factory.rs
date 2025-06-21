use super::{IPluginFactory,IPluginFactory2,IPluginFactory3, IComponent, IAudioProcessor};
use super::factory_flags::FactoryFlags;
use super::pfactory_info::PFactoryInfo;
use super::pclass_info::{PClassInfo, PClassInfo2, PClassInfoW, ClassInfo};
use super::Error;
use windows::core::{GUID, Interface};


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
	pub fn new(factory: IPluginFactory) -> Result<PluginFactory, Error> {
		let mut factory_info = PFactoryInfo::new();

		let hr = unsafe {
			factory.getFactoryInfo(&mut factory_info)
		};
		if hr.is_err() {
			return Err(Error::from_hresult("Failed to get factory info.", hr));
		}

		let factory_2 =factory.cast::<IPluginFactory2>().ok();
		let factory_3: Option<IPluginFactory3> = None;

		Ok(PluginFactory {
			factory,
			factory_2,
			factory_3,
			vendor: factory_info.get_vendor(),
			url: factory_info.get_url(),
			email: factory_info.get_email(),
			flags: factory_info.get_flags()
		})
	}

	pub fn create_audio_processor(&self) -> Result<IAudioProcessor, Error> {
		self.create_component::<IAudioProcessor>("Audio Module Class")
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

	fn get_class_info(&self, index: usize) -> Result<ClassInfo, Error> {
		if index >= self.count_classes() {
			Err(Error::from_other("Factory has no associated class"))
		}
		else {
			match &self.factory_3 {
				Some(factory_3) => {
					let mut pclass_info_w = PClassInfoW::new();
					unsafe {
						let hr = factory_3.getClassInfoUnicode(index as i32, &mut pclass_info_w);
						if hr.is_err()
						{
							Err(Error::from_hresult("Failed to get class info (unicode).", hr))
						}
						else
						{
							Ok(ClassInfo::from_class_info_w(&pclass_info_w))
						}
					}
				},
				None => {
					match &self.factory_2 {
						Some(factory_2) => {
							let mut pclass_info_2 = PClassInfo2::new();
							unsafe {
								let hr = factory_2.getClassInfo2(index as i32, &mut pclass_info_2);
								if hr.is_err() {
									Err(Error::from_hresult("Failed to get class info (v2).", hr))
								}
								else {
									Ok(ClassInfo::from_class_info_2(&pclass_info_2))
								}
							}
						},
						None => {
							let mut pclass_info = PClassInfo::new();
							unsafe {
								let hr = self.factory.getClassInfo(index as i32, &mut pclass_info);
								if hr.is_err() {
									Err(Error::from_hresult("Failed to get class info.", hr))
								}
								else {
									Ok(ClassInfo::from_class_info(&pclass_info))
								}
							}
						}
					}
				}
			}
		}
	}

	fn create_component<T: windows::core::Interface>(&self, category: &str) -> Result<T, Error> {
		match self.find_class_info(category)? {
			Some(class_info) => self.create_instance::<T>(&class_info.cid, &T::IID),
			None => Err(Error::from_other(&format!("No class for category {}.", category)))
		}
	}

	fn find_class_info(&self, category: &str) -> Result<Option<ClassInfo>, Error> {
		for c in 0..self.count_classes() {
			let class_info = self.get_class_info(c)?;
			if class_info.category == category {
				return Ok(Some(class_info));
			}
		}
		Ok(None)
	}

	fn create_instance<T>(&self, cid: &GUID, iid: &GUID) -> Result<T, Error> {
		let mut opt_instance : Option<T> = None;

		let hr = unsafe {
			// about the beauty of type-safety...
			self.factory.createInstance(cid, iid, &mut opt_instance as *mut _ as *mut *mut std::ffi::c_void)
		};

		if hr.is_err() {
			return Err(Error::from_hresult("Cannot create instance", hr));
		}

		match opt_instance {
			Some(instance) => Ok(instance),
			None => Err(Error::from_other("Create instance returned null"))
		}
	}
}
