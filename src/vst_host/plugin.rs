use super::{IComponent, IEditController, IAudioProcessor};
use windows::{core::{Error, Interface}, Win32::Foundation::E_FAIL};

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
	}

	pub fn get_edit_controller(&self) -> Result<IEditController, Error> {
		self.edit_controller.clone().ok_or_else(|| Error::from_hresult(E_FAIL))
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
