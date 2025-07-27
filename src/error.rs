use windows::core::HRESULT;

#[derive(Debug)]
enum NestedError {
	System(HRESULT),
	Vst(crate::vst_host::Error),
	AsioLib(asiolib::Error),
}

pub struct Error {
	description: String,
	source: Option<NestedError>,
}

impl Error {
	#[allow(dead_code)]
	pub fn from_hresult(description: &str, hr: HRESULT) -> Self {
		Self {
			description: description.into(),
			source: Some(NestedError::System(hr))
		}
	}

	#[allow(dead_code)]
	pub fn from_windows(description: &str, e: windows::core::Error) -> Self {
		Self {
			description: format!("{}: {}", description, e.message()),
			source: Some(NestedError::System(e.code()))
		}
	}

	pub fn from_other(description: &str) -> Self {
		Self {
			description: description.into(),
			source: None
		}
	}

	fn inner_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match &self.source {
			Some(NestedError::System(hr)) => write!(f, "{} {}", self.description, hr),
			Some(NestedError::Vst(vst)) => write!(f, "{} {}", self.description, vst),
			Some(NestedError::AsioLib(asio)) => write!(f, "{} {}", self.description, asio),
			_ => write!(f, "{}", self.description)
		}
	}
}

impl std::error::Error for Error {
}

impl std::fmt::Debug for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.inner_fmt(f)
	}
}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.inner_fmt(f)
	}
}

impl From<asiolib::Error> for Error {
	fn from(value: asiolib::Error) -> Self {
		Error {
			description: "ASIO lib raised an error.".into(),
			source: Some(NestedError::AsioLib(value)),
		}
	}
}

impl From<crate::vst_host::Error> for Error {
	fn from(value: crate::vst_host::Error) -> Self {
		Error {
			description: "VST raised an error.".into(),
			source: Some(NestedError::Vst(value)),
		}
	}
}

impl From<windows::core::Error> for Error {
	fn from(value: windows::core::Error) -> Self {
		Error {
			description: value.message(),
			source: Some(NestedError::System(value.code()))
		}
	}
}
