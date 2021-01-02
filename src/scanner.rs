use opencv::core as cv;
use opencv::imgproc as imgproc;
use opencv::highgui;
use opencv::imgcodecs::{ imread, ImreadModes};
use cv::Mat;
use cv::MatTrait;

use std::ffi::c_void;
use dxgcap::DXGIManager;

use crate::configuration::cfg_i32;

fn debug_show(name: &str, mat: &Mat) {
    let wait_time = 5000; // delay ms
    highgui::named_window(name, highgui::WINDOW_AUTOSIZE/* highgui::WINDOW_NORMAL |  highgui::WINDOW_KEEPRATIO */).unwrap();
    highgui::imshow(name, mat).unwrap();
    highgui::wait_key(wait_time).unwrap();
    highgui::destroy_window(name).unwrap();
}

pub fn screenshot() -> Result<Mat, String> {
    let mut manager = DXGIManager::new(300).unwrap();
    let (mut bgra, (width, height)) = manager.capture_frame_components().unwrap();
    let ptr = bgra.as_mut_ptr() as *mut c_void;
    let mat_result = unsafe { Mat::new_rows_cols_with_data(height as i32, width as i32, cv::CV_8UC4, ptr, cv::Mat_AUTO_STEP) };
    let mat = mat_result.expect("Failed to initialize matrix data").clone(); // Deep clone data to avoid dangling pointer
    // debug_show("screenshot", &mat);
    Ok(mat)
}

pub fn capture_and_scan() -> Result<(), String> {
    let screen: cv::Mat = screenshot().expect("Failed to capture screenshot");
    scan(&screen)
}

pub fn scan(screen: &Mat) -> Result<(), String> {
    let mut grey = unsafe { Mat::new_rows_cols(screen.rows(), screen.cols(), cv::CV_8UC1).expect("Failed to initialize matrix") };
    // convert to greyscale
    imgproc::cvt_color(&screen, &mut grey, imgproc::COLOR_BGR2GRAY, 0).unwrap();
    // debug_show(&grey);
    
    // Search buffer size
    let buffer_size = detect_buffer_size(&grey).expect("Failed to detect buffer size");
    println!("Buffer size detected: {}", buffer_size);

    Ok(())
}

fn detect_buffer_size(grey: &Mat) -> Result<i32, String> {
    // Get buffer section
    let buffer_left = cfg_i32("buffer.left");
    let buffer_right = cfg_i32("buffer.right");
    let buffer_top = cfg_i32("buffer.top");
    let buffer_bottom = cfg_i32("buffer.bottom");
    let width = buffer_right - buffer_left;
    let height = buffer_bottom - buffer_top;
    let rect = cv::Rect::new(buffer_left, buffer_top, width, height);
    let buffer = Mat::roi(&grey, rect).unwrap();

    // Match buffer template on thresholded image
    // Load match template
    let buffer_template = imread("assets/images/buffer.png", ImreadModes::IMREAD_GRAYSCALE as i32).expect("File buffer.png not found");
    // Prepare threshold image
    let threshold = 70.0;
    let mut thr_buffer = Mat::default().unwrap();
    imgproc::threshold(&buffer, &mut thr_buffer, threshold, 255.0, imgproc::THRESH_BINARY).unwrap();

    // Find matches, each pixel represents the template similarity from 0 (worst) to 1 (best)
    let mut match_result = Mat::default().unwrap();
    let mask = cv::no_array().unwrap();
    imgproc::match_template(&thr_buffer, &buffer_template, &mut match_result, imgproc::TemplateMatchModes::TM_CCOEFF_NORMED as i32, &mask).unwrap();
    
    // Find maximum spots by threshold and count points above threshold
    let mut thr_match_result = Mat::default().unwrap();
    imgproc::threshold(&match_result, &mut thr_match_result, 0.9, 255.0, imgproc::THRESH_BINARY).unwrap();

    let buffer_size = cv::count_non_zero(&thr_match_result).unwrap();
    Ok(buffer_size)
}

#[test]
fn test_scan() {
    let test_screen = imread("assets/images/test1.png", ImreadModes::IMREAD_UNCHANGED as i32).expect("File test1.png not found");
    // debug_show(&test_screen);
    scan(&test_screen);
}

#[test]
fn test_buffer() {
    let test_screen = imread("assets/images/test1.png", ImreadModes::IMREAD_GRAYSCALE as i32).expect("File test1.png not found");
    let buffer_size = detect_buffer_size(&test_screen).unwrap();
    assert!(buffer_size == 8);
}
