use crate::vst_host::{connection_proxy::ConnectionProxy, thread_check::ThreadCheck, IConnectionPoint, IConnectionPoint_Impl, IPlugFrame, ViewRect, PLATFORM_TYPE_HWND};

use super::{
	IPlugView, IComponent, IEditController, IAudioProcessor, FIDString, Error
};

use windows::{core::Interface, Win32::Foundation::HWND};
use windows_core::ComObject;

pub struct Plugin {
	component: Option<IComponent>,
	edit_controller: Option<IEditController>,
	view: Option<IPlugView>,
	audio_processor: Option<IAudioProcessor>,
	comp_proxy: ComObject<ConnectionProxy>,
	edit_proxy: ComObject<ConnectionProxy>,
}

impl Plugin {
	pub fn new(component: IComponent, edit_controller: IEditController, thread_check: ThreadCheck) -> Result<Plugin, Error> {
		println!("New plugin");
		let (comp_proxy, edit_proxy) = Self::connect_components(&component, &edit_controller, thread_check)?;
		Ok(Plugin {
			component: Some(component),
			edit_controller: Some(edit_controller),
			view: None,
			audio_processor: None,
			comp_proxy,
			edit_proxy
		})
	}

	pub fn create_audio_processor(&mut self) -> Result<(), Error> {
		match self.component.as_ref().unwrap().cast::<IAudioProcessor>() {
			Ok(ap) => {
				self.audio_processor = Some(ap);
				Ok(())
			}
			Err(e) => {
				self.audio_processor = None;
				Err(Error::from_windows("Cannot get IAudioProcessor interface", e))
			}
		}
	}

	pub fn create_view(&mut self, frame: &IPlugFrame, hwnd: &HWND) -> Result<(), Error> {
		// Important: the view type must be zero-terminated, so use a c"..." literal!
		match self.edit_controller.as_ref() {
			Some(ec) =>	unsafe {
				let raw_ptr = ec.createView(c"editor".as_ptr() as FIDString);

				if raw_ptr.is_null() {
					Err(Error::from_other("Cannot create 'editor' view. Method returned null"))
				}
				else {
					let iface : IPlugView = windows_core::Interface::from_raw(raw_ptr as *mut std::ffi::c_void);
					let mut view_rect = ViewRect::default();

					iface.getSize(&mut view_rect as *mut ViewRect).ok()
						.and_then(|_| iface.isPlatformTypeSupported(PLATFORM_TYPE_HWND).ok())
						.and_then(|_| iface.setFrame(std::mem::transmute_copy(frame)).ok())
						.and_then(|_| iface.attached(hwnd.0, PLATFORM_TYPE_HWND).ok())
						.or_else(|e| Err(Error::from_windows("Cannot set IPlugFrame for IPlugView.", e)))
				}
			},
			None => Err(Error::from_other("Plugin has no IEditController"))
		}
	}

	pub fn get_parameter_count(&self) -> usize {
		(match self.edit_controller.as_ref() {
			Some(ec) => unsafe { ec.getParameterCount() },
			None => 0
		}) as usize
	}

	fn connect_components(component: &IComponent, edit_controller: &IEditController, thread_check: ThreadCheck) -> Result<(ComObject<ConnectionProxy>, ComObject<ConnectionProxy>), Error>
	{
		let comp_cp : IConnectionPoint = component.cast()
			.or_else(|e| Err(Error::from_windows("Failed to get connection point for component", e)))?;

		let edit_cp : IConnectionPoint = edit_controller.cast()
			.or_else(|e| Err(Error::from_windows("Failed to get connection point for edit controller", e)))?;

		let comp_obj = ComObject::new(ConnectionProxy::new(comp_cp.clone(), thread_check));
		let edit_obj = ComObject::new(ConnectionProxy::new(edit_cp.clone(), thread_check));

		let comp_proxy : IConnectionPoint = comp_obj.cast().unwrap();
		let edit_proxy : IConnectionPoint = edit_obj.cast().unwrap();

		let hr = unsafe { comp_proxy.connect(std::mem::transmute_copy(&edit_cp)) };

		if hr.is_err() {
			Err(Error::from_hresult("Failed to connect edit controller to component proxy", hr))
		}
		else {
			let hr = unsafe { edit_proxy.connect (std::mem::transmute_copy(&comp_cp)) };

			if hr.is_err() {
				Err(Error::from_hresult("Failed to connect component to edit controller proxy", hr))
			}
			else {
				Ok((comp_obj, edit_obj))
			}
		}
	}
}

impl Drop for Plugin {
	fn drop(&mut self) {
		println!("Dropping plugin");
		if let Some(dst) = self.comp_proxy.get_dst() {
			let _ = unsafe { self.comp_proxy.disconnect(std::mem::transmute_copy(&dst)) };
		}
		if let Some(dst) = self.edit_proxy.get_dst() {
			let _ = unsafe { self.edit_proxy.disconnect(std::mem::transmute_copy(&dst)) };
		}
		if let Some(view) = self.view.take() {
			drop(view);
		}
		if let Some(ap) = self.audio_processor.take() {
			drop(ap);
		}
		if let Some(edc) = self.edit_controller.take() {
			drop(edc);
		}
		if let Some(cmp) = self.component.take() {
			drop(cmp);
		}
	}
}
