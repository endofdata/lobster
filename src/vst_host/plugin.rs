use super::{
	IPlugView, IComponent, IEditController, IAudioProcessor, FIDString, Error
};

use windows::{
	core::Interface,
	Win32::Foundation::E_FAIL
};

pub struct Plugin {
	component: Option<IComponent>,
	edit_controller: Option<IEditController>,
}

impl Plugin {
	pub fn new(component: IComponent, edit_controller: IEditController) -> Plugin {
		Plugin { component: Some(component), edit_controller: Some(edit_controller) }
	}

	pub fn create_audio_processor(&self) -> Result<IAudioProcessor, Error> {
		self.component.as_ref().unwrap().cast::<IAudioProcessor>()
			.or_else(|e| Err(Error::from_windows("Cannot get IAudioProcessor interface", e)))
	}

	pub fn create_view(&self) -> Result<IPlugView, Error> {
		// Important: the view type must be zero-terminated, so use a c"..." literal!
		match self.edit_controller.as_ref() {
			Some(ec) =>	unsafe {
				let raw_ptr = ec.createView(c"editor".as_ptr() as FIDString);

				if raw_ptr != std::ptr::null() {
					//let raw_ptr = option.unwrap();
					let iface : IPlugView = windows_core::Interface::from_raw(raw_ptr as *mut std::ffi::c_void);
					Ok(iface)
				}
				else {
					Err(Error::from_other("Cannot create 'editor' view. Method returned null"))
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

	fn get_edit_controller(&self) -> Result<IEditController, Error> {
		self.edit_controller.clone().ok_or_else(|| Error::from_hresult("Failed to add-ref IEditController", E_FAIL))
	}
}

impl Drop for Plugin {
	fn drop(&mut self) {
		if let Some(edc) = self.edit_controller.take() {
			drop(edc);
		}
		if let Some(cmp) = self.component.take() {
			drop(cmp);
		}
	}
}
