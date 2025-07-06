use libloading::Library;
use windows::core::{IUnknown, Interface, GUID};
use crate::vst_host::connection_proxy::ConnectionProxy;
use crate::vst_host::factory_flags::FactoryFlags;
use crate::vst_host::pclass_info::{ClassInfo, PClassInfo, PClassInfo2, PClassInfoW};
use crate::vst_host::pfactory_info::PFactoryInfo;
use crate::vst_host::plugin::Plugin;
use crate::vst_host::{IConnectionPoint, IEditController, IPluginFactory2, IPluginFactory3, VST_AUDIO_EFFECT_CLASS};

use super::{IPluginFactory, IComponent};
use super::Error;

pub struct ClassInfoIter<'a> {
	factory: &'a PluginLibrary,
	pos: usize,
	max: usize
}

impl<'a> ClassInfoIter<'a> {
	pub fn new(factory: &'a PluginLibrary) -> Self {
		Self {
			factory,
			pos: 0,
			max: factory.count_classes()
		}
	}
}

impl<'a> Iterator for ClassInfoIter<'a> {
	type Item = ClassInfo;

	fn next(&mut self) -> Option<Self::Item> {
		if self.pos < self.max {
			let ci = self.factory.get_class_info(self.pos).ok();
			self.pos += 1;
			ci
		}
		else {
			None
		}
	}
}

pub struct PluginLibrary {
	id: String,
	lib: Option<Library>,
	factory: Option<IPluginFactory>,
	factory_2: Option<IPluginFactory2>,
	factory_3: Option<IPluginFactory3>,
	vendor: String,
	url: String,
	email: String,
	flags: FactoryFlags,
}

impl PluginLibrary {
	pub fn load(path: &str) -> Result<PluginLibrary, Error> {
		unsafe {
			match libloading::Library::new(path) {
				Ok(lib) => PluginLibrary::new(&sha256::digest(path), lib),
				Err(error) => Err(Error::from_other(&format!("Failed to load VST '{}': {:?}", path, error)))
			}
		}
	}

	pub fn new(id: &str, lib: Library) -> Result<PluginLibrary, Error> {
		unsafe {
			if let Ok(init_dll) = lib.get::<unsafe extern "C" fn() -> bool>(b"InitDll") {
				if !init_dll() {
					return Err(Error::from_other("InitDll() failed."));
				}
			}
			match lib.get::<unsafe extern "C" fn() -> Option<IPluginFactory>>(b"GetPluginFactory") {
				Ok(get_factory) => match get_factory() {
					Some(factory) => {
						let mut factory_info = PFactoryInfo::new();
						let hr = factory.getFactoryInfo(&mut factory_info);
						if hr.is_err() {
							return Err(Error::from_hresult("Failed to get factory info.", hr));
						}
						let factory_2 = factory.cast::<IPluginFactory2>().ok();
						let factory_3 = factory.cast::<IPluginFactory3>().ok();

						Ok(PluginLibrary {
							id: id.to_string(),
							lib: Some(lib),
							factory: Some(factory),
							factory_2,
							factory_3,
							vendor: factory_info.get_vendor(),
							url: factory_info.get_url(),
							email: factory_info.get_email(),
							flags: factory_info.get_flags()
						})
					},
					None => Err(Error::from_other("VST.GetPluginFactory() returned null."))
				}
				Err(error) => Err(Error::from_other(&format!("Missing entry point 'GetPluginFactory' in VST: {:?}", error)))
			}
		}
	}

	pub fn get_id(&self) -> &str {
		&self.id
	}

	pub fn get_vendor(&self) -> &str {
		&self.vendor
	}

	pub fn get_url(&self) -> &str {
		&self.url
	}

	pub fn get_email(&self) -> &str {
		&self.email
	}

	pub fn get_flags(&self) -> FactoryFlags {
		self.flags
	}

	pub fn get_class_infos<'a>(&'a self) -> ClassInfoIter<'a> {
		ClassInfoIter::new(self)
	}

	pub fn create_plugin(&self, context: IUnknown) -> Result<Plugin, Error> {
		let raw_context = context.as_raw() as *const IUnknown;

		let component = self.create_component_by_category::<IComponent>(VST_AUDIO_EFFECT_CLASS)?;
		let hr = unsafe { component.initialize(raw_context) };

		if hr.is_err() {
			Err(Error::from_hresult("Failed to initialize component", hr))
		}
		else {
			component.cast::<IEditController>()
				.or_else(|_| {
					let mut class_id = GUID::zeroed();
					let hr = unsafe { component.getControllerClassId(&mut class_id) };

					if hr.is_err() {
						Err(Error::from_hresult("Could not get controller class id", hr))
					}
					else {
						self.create_instance::<IEditController>(&class_id, &IEditController::IID)
					}
				})
				.and_then(|edit_controller| {
					let hr = unsafe { edit_controller.initialize(raw_context) };

					if hr.is_err() {
						Err(Error::from_hresult("Could not initialize IEditController", hr))
					}
					else {
						Self::connect_components(&component, &edit_controller)?;
						Ok(Plugin::new(component, edit_controller))
					}
				})
		}
	}

	fn connect_components(component: &IComponent, edit_controller: &IEditController) -> Result<(), Error>
	{
		let comp_cp : IConnectionPoint = component.cast()
			.or_else(|e| Err(Error::from_windows("Failed to get connection point for component", e)))?;

		let edit_cp : IConnectionPoint = edit_controller.cast()
			.or_else(|e| Err(Error::from_windows("Failed to get connection point for edit controller", e)))?;

		let comp_proxy : IConnectionPoint = ConnectionProxy::new(comp_cp.clone()).into();
		let edit_proxy : IConnectionPoint = ConnectionProxy::new(edit_cp.clone()).into();

		let hr = unsafe { comp_proxy.connect(edit_cp.as_raw() as *const IConnectionPoint) };

		if hr.is_err() {
			Err(Error::from_hresult("Failed to connect edit controller to component proxy", hr))
		}
		else {
			let hr = unsafe { edit_proxy.connect (comp_cp.as_raw() as *const IConnectionPoint) };

			if hr.is_err() {
				Err(Error::from_hresult("Failed to connect component to edit controller proxy", hr))
			}
			else {
				Ok(())
			}
		}
	}

	fn count_classes(&self) -> usize {
		unsafe {
			self.get_factory().countClasses() as usize
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
								let hr = self.get_factory().getClassInfo(index as i32, &mut pclass_info);
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

	fn create_component_by_category<T: windows::core::Interface>(&self, category: &str) -> Result<T, Error> {
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

	fn get_factory(&self) -> &IPluginFactory {
		self.factory.as_ref().unwrap()
	}

	fn create_instance<T>(&self, cid: &GUID, iid: &GUID) -> Result<T, Error> {
		let mut opt_instance : Option<T> = None;

		let hr = unsafe {
			// about the beauty of type-safety...
			self.get_factory().createInstance(cid, iid, &mut opt_instance as *mut _ as *mut *mut std::ffi::c_void)
		};

		if hr.is_err() {
			return Err(Error::from_hresult("Cannot create instance", hr));
		}

		match opt_instance {
			Some(instance) => Ok(instance),
			None => Err(Error::from_other("Create instance returned null"))
		}
	}

	fn close(&mut self) -> Result<(), Error> {
		match self.lib.take() {
			Some(lib) => {
				unsafe {
					let opt_method : Result<libloading::Symbol<unsafe extern "C" fn()>, libloading::Error> = lib.get(b"ExitDll");
					match opt_method {
						Ok(exit_dll) => exit_dll(),
						// method is optional
						Err(_) => ()
					}
				}
				match lib.close() {
					Ok(()) => Ok(()),
					Err(e) => Err(Error::from_other(&format!("Error closing library: {:?}", e)))
				}
			},
			None => Ok(())
		}
	}
}

impl Drop for PluginLibrary {
	fn drop(&mut self) {
		println!("Dropping plugin factories");

		if let Some(f3) = self.factory_3.take() {
			drop(f3);
		}
		if let Some(f2) = self.factory_2.take() {
			drop(f2);
		}
		if let Some(f) = self.factory.take() {
			drop(f);
		}

		println!("Unloading plugin library");

		self.close().unwrap();
	}
}
