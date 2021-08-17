fn main() {
    windows::build!(
        Windows::Win32::{
            Foundation::*,
            Graphics::Gdi::{GetDC, ValidateRect},
            System::Diagnostics::Debug::GetLastError,
            System::LibraryLoader::GetModuleHandleA,
            UI::WindowsAndMessaging::*,
        },
    );
}
