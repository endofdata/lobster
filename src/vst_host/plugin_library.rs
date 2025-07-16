use libloading::Library;
use windows::core::{IUnknown, Interface, GUID};
use windows_core::ComObject;
use crate::vst_host::connection_proxy::ConnectionProxy;
use crate::vst_host::factory_flags::FactoryFlags;
use crate::vst_host::pclass_info::{ClassInfo, PClassInfo, PClassInfo2, PClassInfoW};
use crate::vst_host::pfactory_info::PFactoryInfo;
use crate::vst_host::plugin::Plugin;
use crate::vst_host::thread_check::ThreadCheck;
use crate::vst_host::{IConnectionPoint, IEditController, IPluginFactory2, IPluginFactory3, VST_AUDIO_EFFECT_CLASS};

use super::{IPluginFactory, IComponent};
use super::Error;

/// Iterator over [ClassInfo] as provided by [PluginLibrary::get_class_infos]
pub struct ClassInfoIter<'a> {
	lib: &'a PluginLibrary,
	pos: usize,
	max: usize
}

impl<'a> ClassInfoIter<'a> {
	/// Constructor
	///
	/// Creates a new instance to enumerate the class infos of a [PlugInLibrary]
	pub fn new(lib: &'a PluginLibrary) -> Self {
		Self {
			lib,
			pos: 0,
			max: lib.count_classes()
		}
	}
}

impl<'a> Iterator for ClassInfoIter<'a> {
	type Item = ClassInfo;

	fn next(&mut self) -> Option<Self::Item> {
		if self.pos < self.max {
			let ci = self.lib.get_class_info(self.pos).ok();
			self.pos += 1;
			ci
		}
		else {
			None
		}
	}
}

/// A VST plugin library
pub struct PluginLibrary {
	id: String,
	lib: Option<Library>,
	factory: Option<IPluginFactory>,
	factory_2: Option<IPluginFactory2>,
	factory_3: Option<IPluginFactory3>,
	vendor: Option<String>,
	url: Option<String>,
	email: Option<String>,
	flags: FactoryFlags,
}

impl PluginLibrary {
	/// Loads the VST plugin from [path]
	///
	/// The unique identifier as accessible from `PluginFactory::get_id()` is set to the [sha256] digest
	/// of the `path` parameter.
	pub fn load(path: &str) -> Result<Self, Error> {
		unsafe {
			match libloading::Library::new(path) {
				Ok(lib) => Self::new(&sha256::digest(path), lib),
				Err(error) => Err(Error::from_other(&format!("Failed to load VST '{}': {:?}", path, error)))
			}
		}
	}

	/// Contructor
	///
	/// Creates a new instance for the given *lib*. This will call the optional _InitDll_ method, if
	/// provided and the _GetPluginFactory_ method to create the [IPluginFactory] interface. The
	/// plugin factory information (vendor, url, email and flags) is extracted and can be accessed
	/// by the corresponding *get_** methods, i.e. *PluginLibrary::get_vendor()*. Finally, the optional
	/// extended factory interfaces [IPluginFactory2] and [IPluginFactory3] are queried.
	pub fn new(id: &str, lib: Library) -> Result<Self, Error> {
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

	/// Gets a unique identifier
	///
	/// For instances created by `load(...)` this is the [sha256] digest of the library path.
	/// Instances created by `new(...)` method may use an arbitrary identifier chosen by the caller.
	pub fn get_id(&self) -> &str {
		&self.id
	}

	/// Gets the vendor's name from plugin factory info
	pub fn get_vendor(&self) -> Option<&str> {
		self.vendor.as_deref()
	}

	/// Gets the vendor's URL from plugin factory info
	pub fn get_url(&self) -> Option<&str> {
		self.url.as_deref()
	}

	/// Gets the vendor's email from plugin factory info
	pub fn get_email(&self) -> Option<&str> {
		self.email.as_deref()
	}

	/// Gets the flags from plugin factory info
	pub fn get_flags(&self) -> FactoryFlags {
		self.flags
	}

	/// Gets the class informations provided by the plugin factory
	///
	/// The iteration returns items from the highest available plugin factory interface version
	/// implemented by the library.
	pub fn get_class_infos<'a>(&'a self) -> ClassInfoIter<'a> {
		ClassInfoIter::new(self)
	}

	pub fn create_plugin(&self, context: IUnknown, fx_clsid: &Option<GUID>, thread_check: ThreadCheck) -> Result<Plugin, Error> {
		let raw_context : *const IUnknown = unsafe { std::mem::transmute_copy(&context) };

		let component = match fx_clsid {
			Some(id) => self.create_instance::<IComponent>(id),
			None => self.create_component_by_category::<IComponent>(VST_AUDIO_EFFECT_CLASS)
		}?;
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
						self.create_instance::<IEditController>(&class_id)
					}
				})
				.and_then(|edit_controller| {
					let hr = unsafe { edit_controller.initialize(raw_context) };

					if hr.is_err() {
						Err(Error::from_hresult("Could not initialize IEditController", hr))
					}
					else {
						Plugin::new(component, edit_controller, thread_check)
					}
				})
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
			Some(class_info) => self.create_instance::<T>(&class_info.cid),
			None => Err(Error::from_other(&format!("No class for category {}.", category)))
		}
	}

	fn find_class_info(&self, category: &str) -> Result<Option<ClassInfo>, Error> {
		for c in 0..self.count_classes() {
			if let Ok(class_info) = self.get_class_info(c) {
				if class_info.is_category(category) {
					return Ok(Some(class_info));
				}
			}
		}
		Ok(None)
	}

	fn get_factory(&self) -> &IPluginFactory {
		self.factory.as_ref().unwrap()
	}

	fn create_instance<T: Interface>(&self, cid: &GUID) -> Result<T, Error> {
		let mut opt_instance : Option<T> = None;

		let hr = unsafe {
			// about the beauty of type-safety...
			self.get_factory().createInstance(cid, &T::IID, &mut opt_instance as *mut _ as *mut *const std::ffi::c_void)
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

#[cfg(test)]
mod test {
	use windows::Win32::Foundation::E_NOTIMPL;
	use windows_core::{implement, ComObject, GUID, HRESULT};
	use crate::vst_host::{str_conv::StrConv, Error, IEditController, IHostApplication, IHostApplication_Impl, IPlugView, String128, STRING_128_SIZE};

	use super::PluginLibrary;

	const LIBRARY_PATH : &str = "C:\\Program Files\\Common Files\\VST3\\Unfiltered Audio Indent.vst3";

	#[implement(IHostApplication)]
	struct DummyHost {
		name: String
	}

	impl DummyHost {
		pub fn new(name: &str) -> Self {
			Self { name: name.into() }
		}
	}

	impl IHostApplication_Impl for DummyHost_Impl {
		unsafe fn getName(&self, name: String128) -> i32 {
			StrConv::str_to_w_str(&self.name, name, STRING_128_SIZE, true) as i32
		}

		unsafe fn createInstance(&self, _cid: *const GUID, _iid: *const GUID, _ppv: *mut *const std::ffi::c_void) -> HRESULT {
			E_NOTIMPL
		}
	}
}