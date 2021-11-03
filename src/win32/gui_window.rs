use std::{collections::HashMap, ffi::CString};

use windows::{
    runtime::*,
    Win32::{
        Foundation::*, Graphics::Gdi::*, System::LibraryLoader::GetModuleHandleA,
        UI::WindowsAndMessaging::*,
    },
};

use super::utils::*;

pub trait Paintable {
    fn paint(&self, hdc: HDC) -> std::result::Result<(), String>;
}

pub struct GuiWindowClass<'a> {
    class_name_cstr: CString,
    class_name: String,
    wc: WNDCLASSEXA,

    windows: HashMap<isize, GuiWindow<'a>>,
}

impl GuiWindowClass<'_> {
    pub fn new(class_name: &str) -> Self {
        let class_name_cstr = CString::new(class_name).expect("CString::new failed");
        let wc =
            Self::init_window_class(&class_name_cstr).expect("Failed to initialize window class");
        Self {
            class_name_cstr,
            class_name: class_name.to_owned(),
            wc,
            windows: Default::default(),
        }
    }

    fn init_window_class(class_name_cstr: &CString) -> std::result::Result<WNDCLASSEXA, String> {
        let instance = unsafe { GetModuleHandleA(None) };
        if let Err(err) = instance.ok() {
            return Err(err.to_string());
        };

        let wc = WNDCLASSEXA {
            cbSize: std::mem::size_of::<WNDCLASSEXA>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(Self::wndproc),
            hInstance: instance,
            // lpszClassName: self.lp_class_name,
            lpszClassName: PSTR(class_name_cstr.as_ptr() as _),
            ..Default::default()
        };

        let atom = unsafe { RegisterClassExA(&wc) };
        if atom == 0 {
            return Err("RegisterClassExA failed".to_string());
        }

        Ok(wc)
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
                    // TODO: this cast is valid only if Self is not dropped
                    let this = GetWindowLong(window, GWLP_USERDATA) as *mut Self;
                    if !this.is_null() {
                        let hdc = BeginPaint(window, &mut ps);
                        if let Some(window) = (*this).windows.get(&window.0) {
                            window.paint(hdc);
                        }
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

    pub fn create_window(&mut self, width: i32, height: i32) -> Result<&GuiWindow> {
        let mut window = GuiWindow::new(width, height);
        window.init(&self.class_name)?;

        let hwnd = window.hwnd.0;
        // Move and register window
        self.windows.insert(window.hwnd.0, window);

        // Return a reference
        let windows_ref = self.windows.get(&hwnd).unwrap();
        Ok(windows_ref)
    }
}

pub struct GuiWindow<'a> {
    pub hwnd: HWND,
    pub width: i32,
    pub height: i32,

    pub painter: Option<&'a dyn FnMut(usize)>,
}

impl Paintable for GuiWindow<'_> {
    fn paint(&self, hdc: HDC) -> std::result::Result<(), String> {
        Ok(())
    }
}

impl<'a> GuiWindow<'a> {
    pub fn new(width: i32, height: i32) -> Self {
        Self {
            hwnd: HWND(0),
            width,
            height,
            painter: None,
        }
    }

    #[allow(non_snake_case)]
    pub fn init(&mut self, class_name: &str) -> Result<()> {
        let lpClassName = PSTR(class_name.to_owned().as_mut_ptr());
        let lpWindowName = class_name.to_owned() + " overlay window";
        // https://docs.microsoft.com/en-us/windows/win32/winmsg/extended-window-styles
        // WS_EX_LAYERED makes window invisible
        let dwExStyle = WS_EX_NOACTIVATE | WS_EX_TRANSPARENT | WS_EX_TOPMOST; // | WS_EX_LAYERED;
                                                                              // https://docs.microsoft.com/en-us/windows/win32/winmsg/window-styles
        let dwStyle = WS_DISABLED;
        // let dwStyle = WS_TILEDWINDOW; // FIXME:
        let x = 0;
        let y = 0;
        let nWidth = self.width;
        let nHeight = self.height;
        let hWndParent = None;
        let hMenu = None;
        let hInstance = HINSTANCE::default(); // self.window_class.wc.hInstance;
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
        }
        .ok()?;

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

    pub fn run(&self) -> Result<()> {
        let mut message = MSG::default();

        loop {
            unsafe {
                GetMessageA(&mut message, None, 0, 0);
                if message.message == WM_QUIT {
                    return Ok(());
                }
                DispatchMessageA(&message);
            }
        }
    }

    pub fn show(&self) {
        unsafe { ShowWindow(self.hwnd, SW_SHOWNOACTIVATE) };
    }

    pub fn hide(&self) {
        unsafe { ShowWindow(self.hwnd, SW_HIDE) };
    }

    pub fn set_painter(&mut self, painter: &'a dyn FnMut(usize)) {
        self.painter = Some(painter)
    }
}

// TESTS
#[cfg(test)]
mod tests {
    // use std::time::Duration;

    use super::*;

    #[test]
    fn it_creates_window() {
        let mut class = GuiWindowClass::new("Test window class");
        let window = class.create_window(300, 300).unwrap();
        window.show();
        window.run().unwrap();

        // Uncomment me to show window for some time, otherwise test will exit immediately
        // std::thread::sleep(Duration::from_millis(2000));
    }
}
