fn main() {
    windows::build!(
        Windows::Win32::{
            Foundation::*,
            Graphics::Gdi::{BeginPaint, BITMAP, BITMAPFILEHEADER, BITMAPINFOHEADER, EndPaint, GetDC, ReleaseDC, StretchDIBits, PAINTSTRUCT},
            System::Diagnostics::Debug::GetLastError,
            System::LibraryLoader::GetModuleHandleA,
            UI::WindowsAndMessaging::*,
        }
    );
}
