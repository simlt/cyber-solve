use opencv::core as cv;

pub(crate) struct Color {
    r: i32,
    g: i32,
    b: i32,
    a: i32,
}

impl Color {
    pub(crate) fn rgba(r: i32, g: i32, b: i32, a: i32) -> Color {
        Color { r, g, b, a }
    }
    pub(crate) fn to_bgra(&self) -> cv::Scalar {
        cv::Scalar::new(self.b as f64, self.g as f64, self.r as f64, self.a as f64)
    }
}

pub(crate) fn lerp_i(start: i32, stop: i32, t: f64) -> i32 {
    start + ((stop - start) as f64 * t).round() as i32
}
pub(crate) fn lerp_f(start: f32, stop: f32, t: f64) -> f64 {
    start as f64 + (stop - start) as f64 * t
}
