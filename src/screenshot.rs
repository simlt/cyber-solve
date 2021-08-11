use opencv::prelude::*;

#[cfg(target_os = "windows")]
pub(crate) fn screenshot() -> Result<Mat, String> {
    use dxgcap::DXGIManager;
    use std::ffi::c_void;
    use opencv::core as cv;
    
    let mut manager = DXGIManager::new(300).unwrap();
    let (mut bgra, (width, height)) = manager.capture_frame_components().unwrap();
    let ptr = bgra.as_mut_ptr() as *mut c_void;
    let mat_result = unsafe {
        Mat::new_rows_cols_with_data(
            height as i32,
            width as i32,
            cv::CV_8UC4,
            ptr,
            cv::Mat_AUTO_STEP,
        )
    };
    let mat = mat_result
    .expect("Failed to initialize matrix data")
    .clone(); // Deep clone data to avoid dangling pointer
    // debug_show("screenshot", &mat);
    Ok(mat)
}

#[cfg(not(target_os = "windows"))]
pub(crate) fn screenshot() -> Result<Mat, String> {
    // Comment the following line to use a test image instead
    panic!("Unsupported platform for screenshot");

    use opencv::imgcodecs::{imread, ImreadModes};

    let screen = imread(
        "assets/images/test_6x6.png",
        ImreadModes::IMREAD_UNCHANGED as i32,
    )
    .expect("File test_6x6.png not found");
    Ok(screen)
}
