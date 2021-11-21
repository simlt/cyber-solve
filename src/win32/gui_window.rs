use std::collections::HashMap;

use windows::{
    core::*,
    Win32::{
        Foundation::*, Graphics::Gdi::*, System::LibraryLoader::GetModuleHandleA,
        UI::WindowsAndMessaging::*,
    },
};

use super::utils::*;

pub trait Paintable {
    fn paint(&self, ps: &mut PAINTSTRUCT) -> std::result::Result<(), String>;
}

pub struct GuiWindowClass {
    class_name: String,
    wc: WNDCLASSA,
    windows: HashMap<isize, Box<GuiWindow>>,
}

#[allow(non_snake_case)]
impl GuiWindowClass {
    pub fn new(class_name: &str) -> Self {
        let instance = unsafe { GetModuleHandleA(None) };
        debug_assert!(instance.0 != 0);

        let mut pstr = class_name
            .bytes()
            .chain(::std::iter::once(0))
            .collect::<std::vec::Vec<u8>>();
        let wc = WNDCLASSA {
            hCursor: unsafe { LoadCursorW(None, IDC_ARROW) },
            hInstance: instance,
            lpszClassName: PSTR(pstr.as_mut_ptr()),

            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(Self::wndproc),
            ..Default::default()
        };

        let atom = unsafe { RegisterClassA(&wc) };
        debug_assert!(atom != 0);

        Self {
            class_name: class_name.to_owned(),
            wc,
            windows: Default::default(),
        }
    }

    pub fn create_window(
        &mut self,
        width: i32,
        height: i32,
        style: Option<WINDOW_STYLE>,
        ex_style: Option<WINDOW_EX_STYLE>,
    ) -> Result<&Box<GuiWindow>> {
        unsafe {
            let mut window = Box::new(GuiWindow::new(width, height));
            if let Some(style) = style {
                window.style = style;
            }
            if let Some(ex_style) = ex_style {
                window.ex_style = ex_style;
            }

            window.init(&self.class_name, self.wc.hInstance)?;

            let hwnd = window.hwnd;
            self.windows.insert(hwnd.0, window);

            let window_ref = self.windows.get(&hwnd.0).unwrap();
            Ok(window_ref)
        }
    }

    pub fn get_window(&self, hwnd: HWND) -> Option<&Box<GuiWindow>> {
        self.windows.get(&hwnd.0)
    }

    fn run_handler(&self) -> Result<()> {
        let mut message = MSG::default();
        unsafe {
            while GetMessageA(&mut message, HWND(0), 0, 0).into() {
                DispatchMessageA(&mut message);
            }
        }

        Ok(())
    }

    unsafe extern "system" fn wndproc(
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
}

#[derive(Default)]
pub struct GuiWindow {
    pub hwnd: HWND,
    pub width: i32,
    pub height: i32,
    // https://docs.microsoft.com/en-us/windows/win32/winmsg/window-styles
    pub style: WINDOW_STYLE,
    // https://docs.microsoft.com/en-us/windows/win32/winmsg/extended-window-styles
    pub ex_style: WINDOW_EX_STYLE,
}

impl Paintable for GuiWindow {
    fn paint(&self, ps: &mut PAINTSTRUCT) -> std::result::Result<(), String> {
        let hbr = HBRUSH((COLOR_WINDOW.0 + 1).try_into().unwrap());
        unsafe { FillRect(ps.hdc, &ps.rcPaint, hbr) };
        Ok(())
    }
}

#[allow(non_snake_case)]
impl GuiWindow {
    pub fn new(width: i32, height: i32) -> Self {
        Self {
            width,
            height,
            style: WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            ..Default::default()
        }
    }

    pub fn init(&mut self, class_name: &str, hInstance: HINSTANCE) -> Result<()> {
        let lpWindowName = class_name.to_owned() + " window";
        let dwExStyle = self.ex_style;
        let dwStyle = self.style;
        let x = 0;
        let y = 0;
        let nWidth = self.width;
        let nHeight = self.height;
        let hWndParent = None;
        let hMenu = None;
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

        // Synchronous WM_NCCREATE message should set self.hwnd
        debug_assert!(handle == self.hwnd);

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
                    BeginPaint(self.hwnd, ps);
                    self.paint(ps).unwrap();
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
        let window = class.create_window(300, 300, None, None).unwrap();
        window.show();
        window.run().unwrap();
    }

    #[test]
    fn it_creates_window_using_thread() {
        let hwnd = Arc::new(AtomicIsize::new(0));
        let hwnd_clone = hwnd.clone();

        let wnd_thread = std::thread::spawn(move || {
            let mut class = GuiWindowClass::new("Test window class");
            let window = class.create_window(300, 300, None, None).unwrap();
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
