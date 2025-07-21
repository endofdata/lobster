#[rustfmt::skip]
use windows::{
	core::{Result, Error, HSTRING},
	Graphics::SizeInt32,
	Win32::{
		Foundation::{E_FAIL, HWND, LPARAM, LRESULT, RECT, WPARAM},
		UI::WindowsAndMessaging::{
			AdjustWindowRectEx, DefWindowProcW, GetClientRect, GetWindowInfo, GetWindowTextLengthW,
			GetWindowTextW, MessageBoxW, PostQuitMessage, SetWindowPos, ShowWindow,
			HWND_TOP, MB_ICONWARNING, MESSAGEBOX_RESULT, MESSAGEBOX_STYLE,
			SWP_NOACTIVATE, SWP_NOCOPYBITS, SWP_NOMOVE, SW_SHOW, WINDOWINFO
		}
	}
};

pub trait WndBase {
	fn set_handle(&mut self, handle: Option<HWND>);
	fn get_handle(&self) -> Result<HWND>;
	fn on_message(&mut self, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT;

	fn show(&self) {
		if let Ok(handle) = self.get_handle() {
			unsafe { _ = ShowWindow(handle, SW_SHOW) };
		}
	}

	fn get_info(&self) -> Result<WINDOWINFO> {
		let handle = self.get_handle()?;
		let mut info = WINDOWINFO::default();
		unsafe { GetWindowInfo (handle, &mut info) }
		.and_then(|_| Ok(info))
	}

	fn get_title(&self) -> Result<&str>;

	#[allow(dead_code)]
	fn copy_title(&self) -> Result<String> {
		self.get_handle()
		.and_then(|handle| {
			unsafe {
				let length = GetWindowTextLengthW(handle) as usize;
				let mut buffer = Vec::<u16>::with_capacity(length + 1);
				GetWindowTextW(handle, &mut buffer);
				String::from_utf16(&buffer)
				.or_else(|_| Err(Error::from_hresult(E_FAIL)))
			}
		})
	}

	fn show_error<E: std::error::Error>(&self, error: &E) -> Result<MESSAGEBOX_RESULT> {
		self.show_error_with_title_and_style(error, None, None)
	}

	#[allow(dead_code)]
	fn show_error_with_title<E: std::error::Error>(&self, error: &E, title: &str) -> Result<MESSAGEBOX_RESULT> {
		self.show_error_with_title_and_style(error, Some(title), None)
	}

	#[allow(dead_code)]
	fn show_error_with_style<E: std::error::Error>(&self, error: &E, style: MESSAGEBOX_STYLE) -> Result<MESSAGEBOX_RESULT> {
		self.show_error_with_title_and_style(error, None, Some(style))
	}

	fn show_error_with_title_and_style<E: std::error::Error>(&self, error: &E, title: Option<&str>, style: Option<MESSAGEBOX_STYLE>) -> Result<MESSAGEBOX_RESULT> {
		let handle = self.get_handle()?;
		let title = title.and_then(|t| Some(HSTRING::from(t)))
		.unwrap_or_else(|| HSTRING::from(self.get_title().as_deref().unwrap_or_else(|_| "Error")));
		let style = style.unwrap_or(MB_ICONWARNING);
		Ok(unsafe { MessageBoxW(Some(handle), &HSTRING::from(error.to_string()), &title, style) })
	}

	#[allow(dead_code)]
	fn get_client_size(&self) -> Result<SizeInt32> {
		let handle = self.get_handle()?;
		unsafe {
			let mut rect = RECT::default();

			GetClientRect(handle, &mut rect)?;

			Ok(SizeInt32 {
				Width: rect.right - rect.left,
				Height: rect.bottom - rect.top,
			})
		}
	}

	fn set_client_size(&self, new_size: SizeInt32) -> Result<()> {
		let mut client_rect = RECT { left: 0, top: 0, right: new_size.Width, bottom: new_size.Height};
		self.adjust_window_rect(&mut client_rect).and_then(|_| {
		self.get_handle().and_then(|handle|
		unsafe { SetWindowPos (
			handle, Some(HWND_TOP), 0, 0,
			client_rect.right - client_rect.left,
			client_rect.bottom - client_rect.top,
			SWP_NOMOVE | SWP_NOCOPYBITS | SWP_NOACTIVATE) })
		})
	}

	fn adjust_window_rect(&self, client_rect: &mut RECT) -> Result<()> {
		let info = self.get_info()?;
		unsafe { AdjustWindowRectEx (client_rect, info.dwStyle, false, info.dwExStyle) }
	}

	fn def_window_proc(&self, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
		unsafe { DefWindowProcW(self.get_handle().unwrap_or_default(), message, wparam, lparam) }
	}

	fn post_quit_message(exit_code: i32) {
		unsafe { PostQuitMessage(exit_code) };
	}

	#[allow(dead_code)]
	fn as_xy(lparam: LPARAM) -> (isize, isize) {
		(lparam.0 & 0xffff, (lparam.0 >> 16) & 0xffff)
	}
}
