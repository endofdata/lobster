use asiolib::ASIOError;
use windows::core::HRESULT;

#[derive(Clone, Copy)]
#[allow(dead_code)]
pub enum ErrorSource {
	Other,
	System(HRESULT),
	ASIO(ASIOError)
}

impl ErrorSource {
	fn inner_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			ErrorSource::Other => write!(f, "no details"),
			ErrorSource::System(hr) => write!(f, "{:#08X} - {}", hr.0, hr.message()),
			ErrorSource::ASIO(asio) => write!(f, "{:?}", asio)
		}
	}
}

impl std::fmt::Display for ErrorSource {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.inner_fmt(f)
	}
}

impl std::fmt::Debug for ErrorSource {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.inner_fmt(f)
	}
}

#[allow(dead_code)]
pub struct Error {
	source: ErrorSource,
	description: String,
}

#[allow(dead_code)]
impl Error {
	pub fn from_other(description: &str) -> Error {
		Error::from_source(description, ErrorSource::Other)
	}

	pub fn from_hresult(description: &str, hresult: HRESULT) -> Error {
		Error::from_source(description, ErrorSource::System(hresult))
	}

	pub fn from_asio(description: &str, asio: ASIOError) -> Error {
		Error::from_source(description, ErrorSource::ASIO(asio))
	}

	pub fn from_source(description: &str, source: ErrorSource) -> Error {
		Error {
			description: description.into(),
			source,
		}
	}

	fn inner_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{} ({:?}).", self.description, self.source)
	}
}

impl From<asiolib::ErrorSource> for ErrorSource {
	fn from(value: asiolib::ErrorSource) -> Self {
		match value {
			asiolib::ErrorSource::ASIO(asio) => ErrorSource::ASIO(asio),
			asiolib::ErrorSource::System(hr) => ErrorSource::System(hr),
			_ => ErrorSource::Other
		}
	}
}

impl From<asiolib::Error> for Error {
	fn from(value: asiolib::Error) -> Self {
		Error {
			description: format!("{:?}", value),
			source: (*value.get_source()).into()
		}
	}
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
