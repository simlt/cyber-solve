use bindings::Windows::Win32::{
    Foundation::*,
    // System::Diagnostics::Debug::GetLastError,
    UI::WindowsAndMessaging::*,
};

// pub(super) fn unwrap_win32_result<T>(result: Result<T, String>) -> Result<T, String> {
//     if let Err(message) = result {
//         let last_error = unsafe { GetLastError() };
//         return Err(format!("{} {:?}", message, last_error));
//     }
//     return result;
// }

#[allow(non_snake_case)]
#[cfg(target_pointer_width = "32")]
pub(super) unsafe fn SetWindowLong(window: HWND, index: WINDOW_LONG_PTR_INDEX, value: isize) -> isize {
    SetWindowLongA(window, index, value as _) as _
}

#[allow(non_snake_case)]
#[cfg(target_pointer_width = "64")]
pub(super) unsafe fn SetWindowLong(window: HWND, index: WINDOW_LONG_PTR_INDEX, value: isize) -> isize {
    SetWindowLongPtrA(window, index, value)
}

#[allow(non_snake_case)]
#[cfg(target_pointer_width = "32")]
pub(super) unsafe fn GetWindowLong(window: HWND, index: WINDOW_LONG_PTR_INDEX) -> isize {
    GetWindowLongA(window, index) as _
}

#[allow(non_snake_case)]
#[cfg(target_pointer_width = "64")]
pub(super) unsafe fn GetWindowLong(window: HWND, index: WINDOW_LONG_PTR_INDEX) -> isize {
    GetWindowLongPtrA(window, index)
}
