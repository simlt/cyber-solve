use std::default::Default;

use windows::{
    runtime::Handle,
    Win32::{Foundation::*, Graphics::Gdi::*, UI::WindowsAndMessaging::*},
};

use super::gui_window::GuiWindowClass;

pub(crate) struct OverlayWindow<'a> {
    hwnd: HWND,
    width: i32,
    height: i32,

    // Loaded image bitmap (may be NULL)
    bmp_info: BITMAPINFO,
    bmp_pixels: Option<Vec<u8>>,

    window_class: GuiWindowClass<'a>,
    // window: &GuiWindow<'a>,
}

impl<'a> OverlayWindow<'a> {
    fn new(width: i32, height: i32, class_name: &str) -> Self {
        let window_class = GuiWindowClass::new(class_name);
        let mut overlay = Self {
            width,
            height,
            bmp_info: Default::default(),
            bmp_pixels: None,
            hwnd: Default::default(),
            window_class,
        };

        let window = overlay
            .window_class
            .create_window(overlay.width, overlay.height)
            .expect("Failed to initialize GuiWindow");
        overlay.hwnd = window.hwnd;

        overlay
    }

    fn init(&'a mut self) -> &'a mut Self {
        // window.set_painter(&|hdc| self.on_paint(hdc));
        self
    }

    pub(crate) fn show(&self) {
        unsafe { ShowWindow(self.hwnd, SW_SHOWNOACTIVATE) };
    }

    pub(crate) fn hide(&self) {
        unsafe { ShowWindow(self.hwnd, SW_HIDE) };
    }

    pub(crate) fn run(&self) -> Result<(), String> {
        // self.window.run(); // TODO:
        Ok(())
    }

    pub(crate) fn load_bitmap_from_bytes(&mut self, bitmap_bytes: &[u8]) -> Result<(), String> {
        unsafe {
            let hdc = GetDC(self.hwnd);
            if let Err(err) = hdc.ok() {
                return Err(err.to_string());
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
}

// TESTS
#[cfg(test)]
mod tests {
    static FILE_TEST_BMP: &str = "test/sample-bitmap.bmp";

    use std::time::Duration;

    use super::*;

    #[test]
    fn it_creates_overlay_window() {
        let window = OverlayWindow::new(300, 300, "Test");
        window.show();
        window.run().unwrap();

        // Uncomment me to show window for some time, otherwise test will exit immediately
        // std::thread::sleep(Duration::from_millis(2000));
    }

    #[test]
    fn it_loads_bitmap() {
        let wnd_thread = std::thread::spawn(|| {
            let bitmap_bytes = std::fs::read(FILE_TEST_BMP).expect("Cannot read test bitmap file");
            let mut window = OverlayWindow::new(300, 300, "Test");
            // window.init();

            window.load_bitmap_from_bytes(&bitmap_bytes).unwrap();
            window.show();
            window.run().unwrap();
        });

        // Wait for some time, then close window
        std::thread::sleep(Duration::from_millis(2000));
        wnd_thread.join().unwrap();
    }
}
