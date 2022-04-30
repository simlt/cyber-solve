use std::io::Write;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

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

pub trait Window {
    fn show(&self);
    fn hide(&self);
    fn set_painter(&mut self, painter: Box<dyn Paintable>);
}

pub struct GuiWindowClass {
    class_name: String,
    wc: WNDCLASSA,
    windows: HashMap<isize, Rc<RefCell<Box<GuiWindow>>>>,
}

#[allow(non_snake_case)]
impl GuiWindowClass {
    pub fn new(class_name: &str) -> Self {
        let instance = unsafe { GetModuleHandleA(None) };
        debug_assert!(instance.0 != 0);

        let pstr = class_name
            .bytes()
            .chain(::std::iter::once(0))
            .collect::<std::vec::Vec<u8>>();
        let wc = WNDCLASSA {
            hCursor: unsafe { LoadCursorW(None, IDC_ARROW) }.expect("Failed to load LoadCursorW"),
            hInstance: instance,
            lpszClassName: PCSTR(pstr.as_ptr()),

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
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        style: Option<WINDOW_STYLE>,
        ex_style: Option<WINDOW_EX_STYLE>,
        // painter: Option<Box<dyn Paintable>>,
    ) -> Result<HWND> {
        let mut window = Box::new(GuiWindow::new(x, y, width, height));
        if let Some(style) = style {
            window.style = style;
        }
        if let Some(ex_style) = ex_style {
            window.ex_style = ex_style;
        }
        // if let Some(painter) = painter {
        //     window.painter = Some(painter);
        // }

        window.init(&self.class_name, self.wc.hInstance)?;

        let hwnd = window.hwnd;

        let window = Rc::new(RefCell::new(window));
        self.windows.insert(hwnd.0, window);

        Ok(hwnd)
    }

    pub fn get_window(&self, hwnd: HWND) -> Option<&Rc<RefCell<Box<GuiWindow>>>> {
        self.windows.get(&hwnd.0).map(|w| w)
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

pub struct GuiWindow {
    pub hwnd: HWND,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    // https://docs.microsoft.com/en-us/windows/win32/winmsg/window-styles
    pub style: WINDOW_STYLE,
    // https://docs.microsoft.com/en-us/windows/win32/winmsg/extended-window-styles
    pub ex_style: WINDOW_EX_STYLE,

    pub painter: Option<Box<dyn Paintable>>,
}

#[allow(non_snake_case)]
impl GuiWindow {
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self {
            x,
            y,
            width,
            height,
            hwnd: Default::default(),
            style: WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            ex_style: Default::default(),
            painter: None,
        }
    }

    pub fn init(&mut self, class_name: &str, hInstance: HINSTANCE) -> Result<()> {
        let lpWindowName = class_name.to_owned() + " window";
        let dwExStyle = self.ex_style;
        let dwStyle = self.style;
        let x = self.x;
        let y = self.y;
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
        };

        // Synchronous WM_NCCREATE message should set self.hwnd
        debug_assert!(handle == self.hwnd);

        Ok(())
    }

    fn on_paint(&self, ps: &mut PAINTSTRUCT) -> std::result::Result<(), String> {
        if let Some(painter) = self.painter.as_ref() {
            return painter.paint(ps);
        }

        // Default
        let hbr = HBRUSH((COLOR_WINDOW.0 + 1).try_into().unwrap());
        unsafe { FillRect(ps.hdc, &ps.rcPaint, hbr) };
        Ok(())
    }

    pub fn run(&self) -> Result<()> {
        let mut message = MSG::default();

        unsafe {
            while GetMessageA(&mut message, None, 0, 0).into() {
                // writeln!(std::io::stdout(), "Run message {}", message.message).unwrap();
                if message.message == WM_QUIT || message.message == WM_USER {
                    return Ok(());
                }
                DispatchMessageA(&message);
            }
        }

        Ok(())
    }

    pub fn send_quit(hwnd: HWND) {
        unsafe { PostMessageA(hwnd, WM_QUIT, None, None) };
    }

    fn message_handler(&mut self, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        // writeln!(std::io::stdout(), "Message handler for {}", message).unwrap();
        match message {
            WM_DESTROY => {
                unsafe { PostQuitMessage(0) };
                return LRESULT(0);
            }
            WM_PAINT => {
                let ref mut ps = PAINTSTRUCT::default();
                unsafe {
                    BeginPaint(self.hwnd, ps);
                    self.on_paint(ps).unwrap();
                    EndPaint(self.hwnd, ps);
                }
                return LRESULT(0);
            }
            _ => unsafe { DefWindowProcA(self.hwnd, message, wparam, lparam) },
        }
    }
}

impl Window for GuiWindow {
    fn show(&self) {
        // unsafe { ShowWindow(self.hwnd, SW_SHOWNOACTIVATE) };
        unsafe { ShowWindow(self.hwnd, SW_SHOW) };
    }

    fn hide(&self) {
        unsafe { ShowWindow(self.hwnd, SW_HIDE) };
    }

    fn set_painter(&mut self, painter: Box<dyn Paintable>) {
        self.painter = Some(painter);
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
        let hwnd = class.create_window(0, 0, 300, 300, None, None).unwrap();
        let window = class.get_window(hwnd).unwrap().borrow();
        window.show();
        window.run().unwrap();
    }

    #[test]
    fn it_creates_window_using_thread() {
        let hwnd = Arc::new(AtomicIsize::new(0));
        let hwnd_clone = hwnd.clone();

        let wnd_thread = std::thread::spawn(move || {
            let mut class = GuiWindowClass::new("Test window class");
            let hwnd = class.create_window(0, 0, 300, 300, None, None).unwrap();
            hwnd_clone.store(hwnd.0, Ordering::Release);
            let window = class.get_window(hwnd).unwrap().borrow();
            window.show();
            window.run().unwrap();
        });

        // Wait for some time, then close window
        std::thread::sleep(Duration::from_millis(2000));
        GuiWindow::send_quit(HWND(hwnd.load(Ordering::Acquire)));
        wnd_thread.join().unwrap();
    }
}
