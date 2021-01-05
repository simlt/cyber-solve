use leptess::LepTess;
use opencv::core as cv;
use opencv::imgproc;
use opencv::prelude::*;

use crate::configuration::{cfg_f64, cfg_i32};

fn init_leptess() -> LepTess {
    let lt =
        LepTess::new(Some("./assets/tesseract"), "eng").expect("Failed to initialize tesseract");
    return lt;
}

pub fn recognize_cell(cell: &Mat) -> Result<String, String> {
    // Tunable params
    let threshold = cfg_f64("ocr_refine.filter_threshold"); // 110.0;
    let height_border = cfg_i32("ocr_refine.height_border"); // 10;
                                                             // let blur_size = cfg_i32("ocr_refine.blur_size");

    // Refine cell image:  normalize() -> negate() -> threshold() -> extend()

    // Blur image
    // let mut blur_cell = Mat::default().unwrap();
    // let ksize = cv::Size::new(blur_size, blur_size);
    // imgproc::gaussian_blur(&cell, &mut blur_cell, ksize, 0.0, 0.0, cv::BORDER_CONSTANT).unwrap();
    // Unsharp image
    // let mut unsharp_cell = Mat::default().unwrap();
    // cv::add_weighted(&normalized_cell, 1.6, &blur_cell, -0.6, 0.0, &mut unsharp_cell, -1).unwrap();
    // // (image, 1.5, gaussian_3, -0.5, 0, image)
    // debug_show("unsharp", &unsharp_cell);

    // Normalize image
    let mut normalized_cell = Mat::default().unwrap();
    let no_mask = cv::no_array().unwrap();
    cv::normalize(
        &cell,
        &mut normalized_cell,
        255.0,
        0.0,
        cv::NORM_MINMAX,
        -1,
        &no_mask,
    )
    .unwrap();

    // Make binary image and invert colors
    let mut thr_cell = Mat::default().unwrap();
    imgproc::threshold(
        &normalized_cell,
        &mut thr_cell,
        threshold,
        255.0,
        imgproc::THRESH_BINARY_INV,
    )
    .unwrap();
    // debug_show("thr", &thr_cell);

    // Extend top-bottom border
    let white = cv::Scalar::new(255.0, 255.0, 255.0, 255.0);
    let mut border_cell = Mat::default().unwrap();
    cv::copy_make_border(
        &thr_cell,
        &mut border_cell,
        height_border,
        height_border,
        0,
        0,
        cv::BORDER_CONSTANT,
        white,
    )
    .unwrap();

    let mut buffer = cv::Vector::new();
    opencv::imgcodecs::imencode(".png", &border_cell, &mut buffer, &cv::Vector::new()).unwrap();

    let text = recognize_text(&buffer.to_vec())?;
    // println!("{} | ", text);
    // print!("{} | ", text);

    Ok(text)
}

pub fn recognize_text(buf: &Vec<u8>) -> Result<String, String> {
    let mut lt = init_leptess();
    let dpi = 70;
    lt.set_image_from_mem(&buf).unwrap();
    lt.set_source_resolution(dpi);

    let text = lt.get_utf8_text().unwrap().trim_end().to_uppercase();

    Ok(text)
}
