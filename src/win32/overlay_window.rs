use std::ffi::CString;

use bindings::Windows::Win32::{
    Foundation::*, Graphics::Gdi::*, System::Diagnostics::Debug::GetLastError,
    System::LibraryLoader::GetModuleHandleA, UI::WindowsAndMessaging::*,
};

fn unwrap_win32_result<T>(result: Result<T, String>) -> Result<T, String> {
    if let Err(message) = result {
        let last_error = unsafe { GetLastError() };
        return Err(format!("{} {:?}", message, last_error));
    }
    return result;
}

pub(crate) struct OverlayWindow {
    hwnd: HWND,
    class_name: String,
    class_name_cstr: CString,
    width: i32,
    height: i32,

    // Loaded image bitmap (may be NULL)
    bmp_info: BITMAPINFO,
    bmp_pixels: Option<Vec<u8>>,
}

impl OverlayWindow {
    fn new(width: i32, height: i32, class_name: &str) -> Self {
        let class_name_cstr = CString::new(class_name).expect("CString::new failed");
        Self {
            hwnd: HWND(0),
            width,
            height,
            class_name: class_name.to_owned(),
            class_name_cstr,
            bmp_info: BITMAPINFO::default(),
            bmp_pixels: None,
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
        self.hwnd = handle;

        // Store self instance pointer for wndproc
        unsafe {
            SetWindowLong(self.hwnd, GWLP_USERDATA, self as *mut Self as _);
        }

        // TODO: how to call run in the thread
        // std::thread::spawn(|| {
        //     unsafe { self.run().unwrap() };
        // });

        Ok(())
    }

    pub(crate) unsafe fn run(&self) -> Result<(), String> {
        let mut message = MSG::default();

        loop {
            GetMessageA(&mut message, None, 0, 0);
            if message.message == WM_QUIT {
                return Ok(());
            }
            DispatchMessageA(&message);
        }
    }

    pub(crate) fn show(&self) {
        unsafe { ShowWindow(self.hwnd, SW_SHOWNOACTIVATE) };
    }

    pub(crate) fn hide(&self) {
        unsafe { ShowWindow(self.hwnd, SW_HIDE) };
    }

    pub(crate) fn load_bitmap_from_bytes(&mut self, bitmap_bytes: &[u8]) -> Result<(), String> {
        unsafe {
            let hdc = GetDC(self.hwnd);
            if hdc.is_null() {
                return Err("GetDC failed".to_string());
            }

            // Read header info
            let bmp_file_size = std::mem::size_of::<BITMAPFILEHEADER>();
            let bmp_info_size = std::mem::size_of::<BITMAPINFOHEADER>();
            let (bmp_file, rest) = bitmap_bytes.split_at(bmp_file_size);
            let (bmp_info, rest) = rest.split_at(bmp_info_size);
            let bmp_file_header: BITMAPFILEHEADER = std::ptr::read(bmp_file.as_ptr() as *const _);
            let bmi_header: BITMAPINFOHEADER = std::ptr::read(bmp_info.as_ptr() as *const _);
            let bmi_colors: RGBQUAD = std::ptr::read(rest.as_ptr() as *const _);
            let bmi_pixels = &bitmap_bytes[bmp_file_header.bfOffBits as usize..];

            let bmp_info = BITMAPINFO {
                bmiHeader: bmi_header,
                bmiColors: [bmi_colors],
            };

            // Store loaded bitmap data into Self
            self.bmp_info = bmp_info;
            self.bmp_pixels = Some(bmi_pixels.to_owned());

            ReleaseDC(self.hwnd, hdc);

            Ok(())
        }
    }

    fn on_paint(&self, hdc: HDC) -> Result<(), String> {
        // Get the client area for size calculation.
        let mut client_rect = RECT::default();
        let result = unsafe { GetClientRect(self.hwnd, &mut client_rect) };
        if !result.as_bool() {
            return Err("GetClientRect failed".to_string());
        }

        let bmp_info = &self.bmp_info;
        let bmi_pixels = match &self.bmp_pixels {
            Some(pixels) => pixels,
            None => return Ok(()),
        };

        let result = unsafe {
            StretchDIBits(
                hdc,
                0,
                0,
                client_rect.right - client_rect.left,
                client_rect.bottom - client_rect.top,
                0,
                0,
                bmp_info.bmiHeader.biWidth,
                bmp_info.bmiHeader.biHeight,
                bmi_pixels.as_ptr() as *const _,
                bmp_info,
                DIB_RGB_COLORS,
                SRCCOPY,
            )
        };
        if result == 0 {
            return Err("StretchDIBits failed".to_string());
        }

        Ok(())
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
                    let mut ps = PAINTSTRUCT::default();
                    let this = GetWindowLong(window, GWLP_USERDATA) as *mut Self;
                    if !this.is_null() {
                        let hdc = BeginPaint(window, &mut ps);
                        unwrap_win32_result((*this).on_paint(hdc)).unwrap();
                        EndPaint(window, &ps);
                    }
                    LRESULT(0)
                }
                WM_DESTROY => {
                    PostQuitMessage(0);
                    LRESULT(0)
                }
                _ => DefWindowProcA(window, message, wparam, lparam),
            }
        }
    }
}

#[allow(non_snake_case)]
#[cfg(target_pointer_width = "32")]
unsafe fn SetWindowLong(window: HWND, index: WINDOW_LONG_PTR_INDEX, value: isize) -> isize {
    SetWindowLongA(window, index, value as _) as _
}

#[allow(non_snake_case)]
#[cfg(target_pointer_width = "64")]
unsafe fn SetWindowLong(window: HWND, index: WINDOW_LONG_PTR_INDEX, value: isize) -> isize {
    SetWindowLongPtrA(window, index, value)
}

#[allow(non_snake_case)]
#[cfg(target_pointer_width = "32")]
unsafe fn GetWindowLong(window: HWND, index: WINDOW_LONG_PTR_INDEX) -> isize {
    GetWindowLongA(window, index) as _
}

#[allow(non_snake_case)]
#[cfg(target_pointer_width = "64")]
unsafe fn GetWindowLong(window: HWND, index: WINDOW_LONG_PTR_INDEX) -> isize {
    GetWindowLongPtrA(window, index)
}

// TESTS
#[cfg(test)]
mod tests {
    static FILE_TEST_BMP: &str = "test/sample-bitmap.bmp";

    use std::time::Duration;

    use super::*;

    #[test]
    fn it_creates_window() {
        let mut window = OverlayWindow::new(300, 300, "Test");
        let result = window.init();
        unwrap_win32_result(result).unwrap();
        window.show();
        unsafe { window.run().unwrap() };

        // Uncomment me to show window for some time, otherwise test will exit immediately
        std::thread::sleep(Duration::from_millis(2000));
    }

    #[test]
    fn it_loads_bitmap() {
        let wnd_thread = std::thread::spawn(|| {
            let bitmap_bytes = std::fs::read(FILE_TEST_BMP).expect("Cannot read test bitmap file");
            let mut window = OverlayWindow::new(300, 300, "Test");
            window.init().unwrap();
            let result = window.load_bitmap_from_bytes(&bitmap_bytes);
            unwrap_win32_result(result).unwrap();
            window.show();
            unsafe { window.run().unwrap() };
        });

        // Wait for some time, then close window
        std::thread::sleep(Duration::from_millis(2000));
        wnd_thread.join().unwrap();
    }
}
