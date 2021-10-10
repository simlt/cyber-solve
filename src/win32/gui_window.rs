use std::{collections::HashMap, ffi::CString};

use bindings::{
    Handle,
    Windows::Win32::{
        Foundation::*, Graphics::Gdi::*, System::LibraryLoader::GetModuleHandleA,
        UI::WindowsAndMessaging::*,
    },
};

use super::utils::*;

pub trait Paintable {
    fn paint(&self, hdc: HDC) -> Result<(), String>;
}

pub struct GuiWindowClass<'a> {
    class_name_cstr: CString,
    class_name: String,
    wc: WNDCLASSEXA,

    windows: HashMap<isize, GuiWindow<'a>>,
}

impl<'a> GuiWindowClass<'a> {
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

    fn init_window_class(class_name_cstr: &CString) -> Result<WNDCLASSEXA, String> {
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

    pub fn create_window(&'a self, width: i32, height: i32) -> Result<&GuiWindow, String> {
        let mut window = GuiWindow::new(self, width, height);
        window.init()?;

        // Register windows
        self.windows.insert(window.hwnd.0, window);

        Ok(&window)
    }
}

pub struct GuiWindow<'a> {
    pub window_class: &'a GuiWindowClass<'a>,
    pub hwnd: HWND,
    pub width: i32,
    pub height: i32,

    pub painter: Option<&'a dyn FnMut(usize)>,
}

impl Paintable for GuiWindow<'_> {
    fn paint(&self, hdc: HDC) -> Result<(), String> {
        Ok(())
    }
}

impl<'a> GuiWindow<'a> {
    pub fn new(window_class: &'a GuiWindowClass, width: i32, height: i32) -> Self {
        Self {
            window_class,
            hwnd: HWND(0),
            width,
            height,
            painter: None,
        }
    }

    #[allow(non_snake_case)]
    pub fn init(&mut self) -> Result<(), String> {
        let lpClassName = PSTR(self.window_class.class_name_cstr.as_ptr() as _);
        let lpWindowName = self.window_class.class_name.clone() + " overlay window";
        // https://docs.microsoft.com/en-us/windows/win32/winmsg/extended-window-styles
        // WS_EX_LAYERED makes window invisible
        let dwExStyle = WS_EX_NOACTIVATE | WS_EX_TRANSPARENT | WS_EX_TOPMOST; // | WS_EX_LAYERED;
                                                                              // https://docs.microsoft.com/en-us/windows/win32/winmsg/window-styles
        let dwStyle = WS_DISABLED;
        let dwStyle = WS_TILEDWINDOW; // FIXME:
        let x = 0;
        let y = 0;
        let nWidth = self.width;
        let nHeight = self.height;
        let hWndParent = None;
        let hMenu = None;
        let hInstance = self.window_class.wc.hInstance;
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

        if let Err(err) = handle.ok() {
            return Err(err.to_string());
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

    pub fn run(&self) -> Result<(), String> {
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

    pub fn set_painter(&self, painter: &'a dyn FnMut(usize)) {
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
        let class = GuiWindowClass::new("Test window class");
        let window = class.create_window(300, 300).unwrap();
        window.show();
        window.run().unwrap();

        // Uncomment me to show window for some time, otherwise test will exit immediately
        // std::thread::sleep(Duration::from_millis(2000));
    }
}