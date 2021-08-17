use std::ffi::CString;

use bindings::Windows::Win32::{
    Foundation::*, Graphics::Gdi::ValidateRect, System::LibraryLoader::GetModuleHandleA,
    UI::WindowsAndMessaging::*,
};

// const WS_EX_NOACTIVATE: WINDOW_EX_STYLE = 0x08000000u32;
// const WS_EX_TRANSPARENT: u32 = 0x00000020u32;
// const WS_EX_TOPMOST: u32 = 0x00000008u32;

pub(crate) struct OverlayWindow {
    handle: HWND,
    class_name: String,
    class_name_cstr: CString,
    width: i32,
    height: i32,
}

impl OverlayWindow {
    fn new(width: i32, height: i32, class_name: &str) -> Self {
        let class_name_cstr = CString::new(class_name).expect("CString::new failed");
        Self {
            handle: HWND(0),
            width,
            height,
            class_name: class_name.to_owned(),
            class_name_cstr,
        }
    }

    fn init_window_class(&self) -> Result<WNDCLASSEXA, String> {
        let instance = unsafe { GetModuleHandleA(None) };
        if instance.is_null() {
            return Err("GetModuleHandleA failed".to_string());
        }

        let wc = WNDCLASSEXA {
            // hCursor: LoadCursorW(None, IDC_ARROW),
            cbSize: std::mem::size_of::<WNDCLASSEXA>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(Self::wndproc),
            hInstance: instance,
            // lpszClassName: self.lp_class_name,
            lpszClassName: PSTR(self.class_name_cstr.as_ptr() as _),
            ..Default::default()
        };

        let atom = unsafe { RegisterClassExA(&wc) };
        if atom == 0 {
            return Err("RegisterClassExA failed".to_string());
        }

        Ok(wc)
    }

    #[allow(non_snake_case)]
    pub(crate) fn init(&mut self) -> Result<(), String> {
        let wc = self.init_window_class()?;

        let lpClassName = PSTR(self.class_name_cstr.as_ptr() as _);
        let lpWindowName = self.class_name.to_owned() + " overlay window";
        // https://docs.microsoft.com/en-us/windows/win32/winmsg/extended-window-styles
        // WS_EX_LAYERED makes window invisible
        let dwExStyle = WS_EX_NOACTIVATE | WS_EX_TRANSPARENT | WS_EX_TOPMOST; // | WS_EX_LAYERED;
                                                                              // https://docs.microsoft.com/en-us/windows/win32/winmsg/window-styles
        let dwStyle = WS_DISABLED;
        let x = 0;
        let y = 0;
        let nWidth = self.width;
        let nHeight = self.height;
        let hWndParent = None;
        let hMenu = None;
        let hInstance = wc.hInstance;
        let lpParam = std::ptr::null_mut();
        let handle = unsafe {
            CreateWindowExA(
                dwExStyle,
                lpClassName,
                lpWindowName,
                dwStyle,
                x,
                y,
                nWidth,
                nHeight,
                hWndParent,
                hMenu,
                hInstance,
                lpParam,
            )
        };

        if handle.is_null() {
            return Err("CreateWindowExA failed".to_string());
        }
        self.handle = handle;

        Ok(())
    }

    pub(crate) fn show(&self) {
        unsafe { ShowWindow(self.handle, SW_SHOWNOACTIVATE) };
    }

    pub(crate) fn hide(&self) {
        unsafe { ShowWindow(self.handle, SW_HIDE) };
    }

    #[allow(dead_code)]
    extern "system" fn wndproc(
        window: HWND,
        message: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        unsafe {
            match message as u32 {
                WM_PAINT => {
                    println!("WM_PAINT");
                    ValidateRect(window, std::ptr::null());
                    LRESULT(0)
                }
                WM_DESTROY => {
                    println!("WM_DESTROY");
                    PostQuitMessage(0);
                    LRESULT(0)
                }
                _ => DefWindowProcA(window, message, wparam, lparam),
            }
        }
    }
}

// TESTS
#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bindings::Windows::Win32::System::Diagnostics::Debug::GetLastError;

    use super::*;

    #[test]
    fn it_creates_window() {
        let mut window = OverlayWindow::new(300, 300, "Test");
        let result = window.init();
        if let Err(message) = result {
            let last_error = unsafe { GetLastError() };
            panic!("{} {:?}", message, last_error);
        }
        window.show();

        // Uncomment me to show window for some time, otherwise test will exit immediately
        std::thread::sleep(Duration::from_millis(3000));
    }
}
