use windows::Win32::{Foundation::*, UI::WindowsAndMessaging::*};

#[allow(non_snake_case)]
#[cfg(target_pointer_width = "32")]
pub(super) unsafe fn SetWindowLong(
    window: HWND,
    index: WINDOW_LONG_PTR_INDEX,
    value: isize,
) -> isize {
    SetWindowLongA(window, index, value as _) as _
}

#[allow(non_snake_case)]
#[cfg(target_pointer_width = "64")]
pub(super) unsafe fn SetWindowLong(
    window: HWND,
    index: WINDOW_LONG_PTR_INDEX,
    value: isize,
) -> isize {
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
