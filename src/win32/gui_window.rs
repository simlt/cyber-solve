use std::collections::HashMap;

use windows::{
    runtime::*,
    Win32::{
        Foundation::*,
        Graphics::Gdi::*,
        System::LibraryLoader::{GetModuleHandleA, GetModuleHandleW},
        UI::WindowsAndMessaging::*,
    },
};

use super::utils::*;

pub trait Paintable {
    fn paint(&self, ps: &mut PAINTSTRUCT, hdc: HDC) -> std::result::Result<(), String>;
}

pub struct GuiWindowClass {
    class_name: String,
    wc: WNDCLASSA,

    windows: HashMap<isize, GuiWindow>,
}

#[allow(non_snake_case)]
impl GuiWindowClass {
    pub fn new(class_name: &str) -> Self {
        let wc = Self::init_window_class(class_name).expect("Failed to initialize window class");
        Self {
            class_name: class_name.to_owned(),
            wc,
            windows: Default::default(),
        }
    }

    fn init_window_class(class_name: &str) -> Result<WNDCLASSA> {
        let instance = unsafe { GetModuleHandleA(None) }.ok()?;
        let mut str = class_name
            .bytes()
            .chain(::std::iter::once(0))
            .collect::<std::vec::Vec<u8>>();
        let lpszClassName = PSTR(str.as_mut_ptr());

        let wc = WNDCLASSA {
            hInstance: instance,
            lpszClassName,

            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(Self::wnd_proc),
            ..Default::default()
        };

        let atom = unsafe { RegisterClassA(&wc) };
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
        if message == WM_NCCREATE {
            let cs = lparam.0 as *const CREATESTRUCTA;
            let this = (*cs).lpCreateParams as *mut GuiWindow;
            (*this).hwnd = window;

            SetWindowLong(window, GWLP_USERDATA, this as _);
        } else {
            let window = GetWindowLong(window, GWLP_USERDATA) as *mut GuiWindow;
            if let Some(window) = window.as_mut() {
                return window.message_handler(message, wparam, lparam);
            }
        }

        DefWindowProcA(window, message, wparam, lparam)
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

pub struct GuiWindow {
    pub hwnd: HWND,
    pub width: i32,
    pub height: i32,
}

impl Paintable for GuiWindow {
    fn paint(&self, ps: &mut PAINTSTRUCT, hdc: HDC) -> std::result::Result<(), String> {
        let hbr = HBRUSH((COLOR_WINDOW.0 + 1).try_into().unwrap());
        unsafe { FillRect(hdc, &ps.rcPaint, hbr) };
        Ok(())
    }
}

#[allow(non_snake_case)]
impl GuiWindow {
    pub fn new(width: i32, height: i32) -> Self {
        Self {
            hwnd: HWND(0),
            width,
            height,
        }
    }

    pub fn init(&mut self, class_name: &str, hInstance: HINSTANCE) -> Result<()> {
        let lpWindowName = class_name.to_owned() + " overlay window";
        // https://docs.microsoft.com/en-us/windows/win32/winmsg/extended-window-styles
        // WS_EX_LAYERED makes window invisible
        let dwExStyle = WS_EX_NOACTIVATE | WS_EX_TRANSPARENT | WS_EX_TOPMOST; // | WS_EX_LAYERED;
        // let dwExStyle = Default::default(); // FIXME:

        // https://docs.microsoft.com/en-us/windows/win32/winmsg/window-styles
        let dwStyle = WS_DISABLED;
        // let dwStyle = WS_TILEDWINDOW; // FIXME:
        // let dwStyle = WS_OVERLAPPEDWINDOW | WS_VISIBLE; // FIXME:
        let x = 0;
        let y = 0;
        let nWidth = self.width;
        let nHeight = self.height;
        let hWndParent = None;
        let hMenu = None;
        // let hInstance = None; // self.window_class.wc.hInstance;
        let lpParam = self as *mut _ as _; // Store Self instance pointer for wndproc
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
        // unsafe { ShowWindow(self.hwnd, SW_SHOWNOACTIVATE) };
        unsafe { ShowWindow(self.hwnd, SW_SHOW) };
    }

    pub fn hide(&self) {
        unsafe { ShowWindow(self.hwnd, SW_HIDE) };
    }

    pub fn send_quit(hwnd: HWND) {
        unsafe { PostMessageA(hwnd, WM_QUIT, None, None) };
    }

    fn message_handler(&mut self, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        match message {
            WM_DESTROY => {
                unsafe { PostQuitMessage(0) };
                return LRESULT(0);
            }
            WM_PAINT => {
                let ref mut ps = PAINTSTRUCT::default();
                unsafe {
                    let hdc = BeginPaint(self.hwnd, ps);
                    self.paint(ps, hdc).unwrap();
                    EndPaint(self.hwnd, ps);
                }
                return LRESULT(0);
            }
            _ => unsafe { DefWindowProcA(self.hwnd, message, wparam, lparam) },
        }
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
        let mut class = GuiWindowClass::new("Test window class");
        let window = class.create_window(300, 300).unwrap();
        window.show();
        window.run().unwrap();
    }

    #[test]
    fn it_creates_window_using_thread() {
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
        std::thread::sleep(Duration::from_millis(2000));
        GuiWindow::send_quit(HWND(hwnd.load(Ordering::Acquire)));
        wnd_thread.join().unwrap();
    }
}
