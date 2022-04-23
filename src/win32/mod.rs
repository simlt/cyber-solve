pub(super) mod gui_window;
pub(super) mod overlay_window;
pub(super) mod utils;


macro_rules! rgb {
    ($r:expr, $g:expr, $b:expr) => {{
        $b << 8 | $g << 4 | $r
    }};
}

pub(super) use rgb;