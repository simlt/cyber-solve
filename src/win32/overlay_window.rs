use std::{
    cell::{Ref, RefCell, RefMut},
    default::Default,
    sync::{
        atomic::{AtomicIsize, Ordering, AtomicBool},
        Arc, mpsc,
    },
};

use windows::{
    core::Error,
    Win32::{Foundation::*, Graphics::Gdi::*, UI::WindowsAndMessaging::*},
};

use super::{
    gui_window::{GuiWindow, GuiWindowClass, Paintable, Window},
    rgb,
};

pub(crate) struct OverlayWindow {
    hwnd: HWND,
    x: i32,
    y: i32,
    width: i32,
    height: i32,

    window_class: GuiWindowClass,
}

impl OverlayWindow {
    fn new(x: i32, y: i32, width: i32, height: i32, class_name: &str) -> Self {
        let window_class = GuiWindowClass::new(class_name);
        let mut overlay = Self {
            x,
            y,
            width,
            height,
            hwnd: Default::default(),
            window_class,
        };

        overlay.create_window_and_show();

        overlay
    }

    fn create_window_and_show(&mut self) -> () {
        // WS_EX_LAYERED makes window invisible
        // let ex_style = WS_EX_NOACTIVATE | WS_EX_TRANSPARENT | WS_EX_TOPMOST | WS_EX_LAYERED | WS_EX_NOREDIRECTIONBITMAP;
        let ex_style = WS_EX_NOACTIVATE | WS_EX_TOPMOST | WS_EX_LAYERED;
        let style = WS_POPUP | WS_DISABLED;
        // let style = WS_TILEDWINDOW;
        // let style = WS_OVERLAPPEDWINDOW | WS_VISIBLE;
        let hwnd = self
            .window_class
            .create_window(
                self.x,
                self.y,
                self.width,
                self.height,
                Some(style),
                Some(ex_style),
            )
            .expect("Failed to initialize GuiWindow");
        
        // Set transparency
        // https://docs.microsoft.com/en-us/windows/win32/winmsg/window-features#layered-windows
        unsafe { SetLayeredWindowAttributes(hwnd, rgb!(0, 0, 0), 255, LWA_COLORKEY) }
            .ok()
            .expect("SetLayeredWindowAttributes error");

        self.hwnd = hwnd;

        self.show();
    }

    pub fn load_bitmap(&mut self, bitmap: &[u8]) -> Result<(), String> {
        let painter = OverlayWindowPainter::new_from_bitmap(self.hwnd, bitmap)?;
        let mut window = self.get_window_mut();
        window.set_painter(Box::new(painter));
        Ok(())
    }

    fn get_window(&self) -> Ref<Box<GuiWindow>> {
        self.window_class.get_window(self.hwnd).unwrap().borrow()
    }

    fn get_window_mut(&self) -> RefMut<Box<GuiWindow>> {
        let window = self.window_class.get_window(self.hwnd).unwrap();
        let window = RefCell::borrow_mut(window);
        window
    }

    pub(crate) fn show(&self) {
        self.get_window().show();
    }

    pub(crate) fn hide(&self) {
        self.get_window().hide();
    }

    pub(crate) fn run(&self) -> Result<(), String> {
        if let Err(error) = self.get_window().run() {
            return Err(error.to_string());
        }
        Ok(())
    }
}

struct OverlayWindowPainter {
    hwnd: HWND,
    // Loaded image bitmap (may be NULL)
    bmp_info: BITMAPINFO,
    bmp_pixels: Option<Vec<u8>>,
}

impl OverlayWindowPainter {
    fn new(hwnd: HWND) -> Self {
        Self {
            hwnd,
            bmp_info: Default::default(),
            bmp_pixels: None,
        }
    }

    fn new_from_bitmap(hwnd: HWND, bitmap_bytes: &[u8]) -> Result<Self, String> {
        let mut painter = Self {
            hwnd,
            bmp_info: Default::default(),
            bmp_pixels: None,
        };
        painter.load_bitmap_from_bytes(bitmap_bytes)?;

        Ok(painter)
    }

    fn load_bitmap_from_bytes(&mut self, bitmap_bytes: &[u8]) -> Result<(), String> {
        unsafe {
            let hdc = GetDC(self.hwnd);
            if hdc.is_invalid() {
                return Err(Error::from_win32().message().to_string());
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
}

impl Paintable for OverlayWindowPainter {
    fn paint(&self, ps: &mut PAINTSTRUCT) -> Result<(), String> {
        let hdc = ps.hdc;
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

#[derive(Clone)]
pub struct OverlayController {
    hwnd: Arc<AtomicIsize>,
    is_visible: Arc<AtomicBool>,
    tx: mpsc::Sender<Vec<u8>>,
}

impl OverlayController {
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        let hwnd = Arc::new(AtomicIsize::new(0));
        let is_visible = Arc::new(AtomicBool::new(false));
        let (tx, rx) = mpsc::channel::<Vec<u8>>();

        let controller = Self { hwnd, tx, is_visible };
        let controller_clone = controller.clone();
        let _wnd_thread = std::thread::spawn(move || {
            let mut overlay = OverlayWindow::new(x, y, width, height, "Overlay");
            controller_clone.hwnd.store(overlay.hwnd.0, Ordering::Release);
            loop {
                let bitmap_bytes = rx.recv().unwrap();
                overlay.load_bitmap(&bitmap_bytes).unwrap();
                overlay.show();
                controller_clone.is_visible.store(true, Ordering::Release);
                // writeln!(std::io::stdout(), "#### SHOW ####").unwrap();
                overlay.run().unwrap();
                overlay.hide();
                // writeln!(std::io::stdout(), "#### HIDE ####").unwrap();
            }
        });
        controller
    }

    pub fn load(&self, bitmap_bytes: &[u8]) -> () {
        self.hide();
        self.tx.send(bitmap_bytes.to_owned()).expect("Failed to load bitmap");
    }

    pub fn break_run_thread(&self) {
        let hwnd = HWND(self.hwnd.load(Ordering::Acquire));
        // Send custom WM_USER which will break the window.run inside the thread loop
        unsafe { PostMessageW(hwnd, WM_USER, None, None) };
    }

    pub fn hide(&self) {
        if self.is_visible.swap(false, Ordering::AcqRel) {
            self.break_run_thread();
        }
    }

    pub fn quit(&self) {
        let hwnd = HWND(self.hwnd.load(Ordering::Acquire));
        unsafe { PostMessageW(hwnd, WM_QUIT, None, None) };
    }
}

// TESTS
#[cfg(test)]
mod tests {
    static FILE_TEST_BMP: &str = "test/sample-bitmap.bmp";

    use std::{thread::sleep, time::Duration};

    use super::*;

    #[test]
    fn it_creates_overlay() {
        OverlayWindow::new(0, 0, 300, 300, "Test");
    }

    #[test]
    fn it_loads_bitmap_bytes() {
        let bitmap_bytes = std::fs::read(FILE_TEST_BMP).expect("Cannot read test bitmap file");
        let overlay = OverlayController::new(0, 0, 300, 300);
        overlay.load(&bitmap_bytes);

        sleep(Duration::from_secs(3));
        overlay.quit();
    }

    #[test]
    fn it_cycles_show_hide() {
        let bitmap_bytes = std::fs::read(FILE_TEST_BMP).expect("Cannot read test bitmap file");
        let overlay = OverlayController::new(0, 0, 300, 300);
        overlay.load(&bitmap_bytes);

        sleep(Duration::from_secs(1));
        overlay.hide();

        sleep(Duration::from_secs(1));
        overlay.load(&bitmap_bytes);

        sleep(Duration::from_secs(3));
        overlay.quit();
    }
}
