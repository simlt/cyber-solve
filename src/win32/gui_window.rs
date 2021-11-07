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
            class_name: class_name.to_owned(),
            wc,
            windows: Default::default(),
        }
    }

    fn init_window_class(class_name_cstr: &CString) -> Result<WNDCLASSEXA> {
        let instance = unsafe { GetModuleHandleA(None) }.ok()?;

        let wc = WNDCLASSEXA {
            cbSize: std::mem::size_of::<WNDCLASSEXA>() as u32,
            hInstance: instance,
            lpszClassName: PSTR(class_name_cstr.as_ptr() as _),

            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(Self::wnd_proc),
            ..Default::default()
        };

        let atom = unsafe { RegisterClassExA(&wc) };
        if atom == 0 {
            return Err(Error::from_win32());
        }

        Ok(wc)
    }

    unsafe extern "system" fn wnd_proc(
        window: HWND,
        message: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        let this = GetWindowLong(window, GWLP_USERDATA) as *mut GuiWindow;
        if let Some(this) = this.as_mut() {
            return this.message_handler(message, wparam, lparam);
        }

        DefWindowProcW(window, message, wparam, lparam)
    }

    pub fn create_window(&mut self, width: i32, height: i32) -> Result<&GuiWindow> {
        let mut window = GuiWindow::new(width, height);
        window.init(&self.class_name, self.wc.hInstance)?;

        // Move and register window
        let hwnd = window.hwnd;
        self.windows.insert(hwnd.0, window);

        // Return a reference
        let windows_ref = self.get_window(hwnd).unwrap();
        Ok(windows_ref)
    }

    pub fn get_window(&self, hwnd: HWND) -> Option<&GuiWindow> {
        self.windows.get(&hwnd.0)
    }
}

pub struct GuiWindow<'a> {
    pub hwnd: HWND,
    pub width: i32,
    pub height: i32,

    pub painter: Option<&'a dyn FnMut(usize)>,
}

impl Paintable for GuiWindow<'_> {
    fn paint(&self, _hdc: HDC) -> std::result::Result<(), String> {
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
    pub fn init(&mut self, class_name: &str, hInstance: HINSTANCE) -> Result<()> {
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
        // let hInstance = None; // self.window_class.wc.hInstance;
        let lpParam = std::ptr::null_mut();
        let handle = unsafe {
            CreateWindowExA(
                dwExStyle,
                class_name,
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
        //    unsafe { self.run().unwrap() };
        // });

        Ok(())
    }

    pub fn run(&self) -> Result<()> {
        let mut message = MSG::default();

        unsafe {
            while GetMessageA(&mut message, None, 0, 0).into() {
                if message.message == WM_QUIT {
                    return Ok(());
                }
                TranslateMessage(&message);
                DispatchMessageA(&message);
            }
        }

        Ok(())
    }

    pub fn show(&self) {
        unsafe { ShowWindow(self.hwnd, SW_SHOWNOACTIVATE) };
    }

    pub fn hide(&self) {
        unsafe { ShowWindow(self.hwnd, SW_HIDE) };
    }

    pub fn send_quit(hwnd: HWND) {
        unsafe { PostMessageA(hwnd, WM_QUIT, None, None) };
    }

    pub fn set_painter(&mut self, painter: &'a dyn FnMut(usize)) {
        self.painter = Some(painter)
    }

    fn message_handler(&mut self, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        match message {
            WM_DESTROY => {
                unsafe { PostQuitMessage(0) };
                return LRESULT(0);
            }
            WM_PAINT => {
                let mut ps = PAINTSTRUCT::default();
                unsafe {
                    let hdc = BeginPaint(self.hwnd, &mut ps);
                    self.paint(hdc);
                    EndPaint(self.hwnd, &ps);
                }
                return LRESULT(0);
            }
            _ => {}
        }
        unsafe { DefWindowProcW(self.hwnd, message, wparam, lparam) }
    }
}

// TESTS
#[cfg(test)]
mod tests {
    use std::{
        sync::{
            atomic::{AtomicIsize, Ordering},
            Arc,
        },
        time::Duration,
    };

    use super::*;

    #[test]
    fn it_creates_window() {
        let hwnd = Arc::new(AtomicIsize::new(0));
        let hwnd_clone = hwnd.clone();

        let wnd_thread = std::thread::spawn(move || {
            let mut class = GuiWindowClass::new("Test window class");
            let window = class.create_window(300, 300).unwrap();
            hwnd_clone.store(window.hwnd.0, Ordering::Release);
            window.show();
            window.run().unwrap();
        });

        // Wait for some time, then close window
        std::thread::sleep(Duration::from_millis(1000));
        GuiWindow::send_quit(HWND(hwnd.load(Ordering::Acquire)));
        wnd_thread.join().unwrap();
    }
}
