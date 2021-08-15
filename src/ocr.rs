use std::collections::HashSet;

use leptess::LepTess;
use opencv::core as cv;
use opencv::imgproc;
use opencv::prelude::*;

use crate::configuration::cfg_str_vec;
use crate::configuration::{cfg_f64, cfg_i32};

pub(crate) struct Ocr {
    leptess: LepTess,
}

impl Ocr {
    pub(crate) fn new() -> Ocr {
        let mut leptess = LepTess::new(Some("./assets/tesseract"), "eng")
            .expect("Failed to initialize tesseract");

        // Set character whitelist
        let valid_codes = cfg_str_vec("valid_codes");
        let valid_char_set: HashSet<String> =
            valid_codes.join("").chars().map(String::from).collect();
        let whitelist: String = valid_char_set
            .iter()
            .fold("".to_string(), |acc, char| (acc + &char));
        leptess
            .set_variable(leptess::Variable::TesseditCharWhitelist, &whitelist)
            .expect("Failed to set tesseract whitelist");
        // println!("Tesseract whitelist set to {}", whitelist);

        Ocr { leptess }
    }

    pub(crate) fn recognize_text(&mut self, buf: &Vec<u8>) -> Result<String, String> {
        // Send image to leptess
        self.leptess.set_image_from_mem(&buf).unwrap();

        // Set dpi after image update
        let dpi = 70;
        self.leptess.set_source_resolution(dpi);

        let text = self
            .leptess
            .get_utf8_text()
            .unwrap()
            .trim_end()
            .to_uppercase();
        Ok(text)
    }

    pub fn recognize_cell(&mut self, cell: &Mat) -> Result<String, String> {
        // Tunable params
        let threshold = cfg_f64("opencv.ocr_filter_threshold"); // 110.0;
        let height_border = cfg_i32("opencv.ocr_height_border"); // 10;

        // Refine cell image:  normalize() -> negate() -> threshold() -> extend()

        // Blur image
        // let mut blur_cell = Mat::default();
        // let ksize = cv::Size::new(blur_size, blur_size);
        // imgproc::gaussian_blur(&cell, &mut blur_cell, ksize, 0.0, 0.0, cv::BORDER_CONSTANT).unwrap();
        // Unsharp image
        // let mut unsharp_cell = Mat::default();
        // cv::add_weighted(&normalized_cell, 1.6, &blur_cell, -0.6, 0.0, &mut unsharp_cell, -1).unwrap();
        // // (image, 1.5, gaussian_3, -0.5, 0, image)
        // debug_show("unsharp", &unsharp_cell);

        // Normalize image
        let mut normalized_cell = Mat::default();
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
        let mut thr_cell = Mat::default();
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
        let mut border_cell = Mat::default();
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

        let text = self.recognize_text(&buffer.to_vec())?;
        Ok(text)
    }
}
