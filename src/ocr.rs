
use std::collections::HashMap;

use opencv::core as cv;
use opencv::prelude::*;
use opencv::imgproc;
use leptess::LepTess;

use crate::configuration::cfg_str_vec;

fn init_leptess() -> LepTess {
    let lt = LepTess::new(Some("./assets/tesseract"), "eng").expect("Failed to initialize tesseract");
    return lt;
}

pub fn recognize_cell(cell: &Mat) -> Result<String, String> {
    // Tunable params TODO:
    let threshold = 110.0;
    let height_border = 10;

    // Process cell:  negate() -> sharpen() -> threshold() -> trim(50) -> extend()

    // TODO: sharpen image here to improve recognition of BD <-> BO

    let mut thr_cell = Mat::default().unwrap();
    // Make binary image and invert colors
    imgproc::threshold(&cell, &mut thr_cell, threshold, 255.0, imgproc::THRESH_BINARY_INV).unwrap();

    // Extend top-bottom border
    // let min_height = 50;
    // let current_height = thr_cell.rows();
    // let height_border = ((min_height - current_height) as f32 * 0.5).ceil().max(0.0) as i32;
    let white = cv::Scalar::new(255.0, 255.0, 255.0, 255.0);
    let mut border_cell = Mat::default().unwrap();
    cv::copy_make_border(&thr_cell, &mut border_cell, height_border, height_border, 0, 0, cv::BORDER_CONSTANT, white).unwrap();

    let mut buffer = cv::Vector::new();
    opencv::imgcodecs::imencode(".png", &border_cell, &mut buffer, &cv::Vector::new()).unwrap();

    let text = recognize_text(&buffer.to_vec())?;
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
