use cv::Mat;
use opencv::core as cv;
use opencv::highgui;
use opencv::imgcodecs::{imread, ImreadModes};
use opencv::imgproc;
use opencv::prelude::*;

use std::collections::HashMap;
use std::convert::TryInto;
use std::num::TryFromIntError;

use crate::configuration::{cfg_get, cfg_i32, cfg_str_vec, DaemonCfg};
use crate::ocr::recognize_cell;
use crate::screenshot::*;
use crate::types::*;

// Debug functions
#[allow(dead_code)]
pub(crate) fn debug_show(name: &str, mat: &Mat) {
    let wait_time = 3000; // delay ms
    highgui::named_window(
        name,
        highgui::WINDOW_AUTOSIZE, /* highgui::WINDOW_NORMAL |  highgui::WINDOW_KEEPRATIO */
    )
    .unwrap();
    highgui::imshow(name, mat).unwrap();
    highgui::wait_key(wait_time).unwrap();
    highgui::destroy_window(name).unwrap();
}

#[allow(dead_code)]
fn debug_image() -> Result<Mat, String> {
    let screen = imread(
        "assets/images/test_6x6.png",
        ImreadModes::IMREAD_UNCHANGED as i32,
    )
    .expect("File test_6x6.png not found");
    Ok(screen)
}

#[allow(dead_code)]
fn debug_contours(img: &Mat, rects: &Vec<cv::Rect>) {
    let green_rgba = cv::Scalar::new(0.0, 255.0, 0.0, 255.0);
    let mut contours = Mat::default();
    imgproc::cvt_color(&img, &mut contours, imgproc::COLOR_GRAY2RGBA, 0).unwrap();
    for rect in rects {
        imgproc::rectangle(
            &mut contours,
            rect.to_owned(),
            green_rgba,
            2,
            imgproc::FILLED,
            0,
        )
        .unwrap();
    }
    // Draw debug contours
    debug_show("contours", &contours);
}

pub(crate) fn capture_and_scan() -> Result<Puzzle, String> {
    let screen: cv::Mat = screenshot().expect("Failed to capture screenshot");
    // Use the following line to use debug image instead of screenshot
    // let screen: cv::Mat = debug_image().unwrap();
    let result = scan(&screen);
    result
}

pub(crate) fn scan<'screen, 'puzzle>(screen: &'screen Mat) -> Result<Puzzle, String> {
    let mut grey = unsafe {
        Mat::new_rows_cols(screen.rows(), screen.cols(), cv::CV_8UC1)
            .expect("Failed to initialize matrix")
    };
    // convert to greyscale
    imgproc::cvt_color(&screen, &mut grey, imgproc::COLOR_BGR2GRAY, 0).unwrap();
    // debug_show(&grey);
    // Detect buffer size
    let buffer_size = detect_buffer_size(&grey).expect("Failed to detect buffer size");
    println!("Buffer size detected: {}", buffer_size);

    // Detect grid info
    let grid_info = detect_grid(&grey).expect("Failed to detect grid info");
    println!("Grid size detected: {}x{}", grid_info.rows, grid_info.cols);

    // Process cell data
    let grid_data = process_grid(&grey, &grid_info).expect("Failed to process grid data");
    let grid = PuzzleGrid {
        rows: grid_info.rows,
        cols: grid_info.cols,
        cells: grid_data,
    };
    println!("Grid:\n{}", grid.to_string());

    // Detect and process daemons
    let daemons = scan_daemons(&grey).expect("Failed to process daemon data");

    let puzzle = Puzzle {
        buffer_size,
        grid,
        daemons,
    };
    Ok(puzzle)
}

fn detect_buffer_size(grey: &Mat) -> Result<u32, String> {
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
    let buffer_template = imread(
        "assets/images/buffer.png",
        ImreadModes::IMREAD_GRAYSCALE as i32,
    )
    .expect("File buffer.png not found");
    // Prepare threshold image
    let threshold = 70.0;
    let mut thr_buffer = Mat::default();
    imgproc::threshold(
        &buffer,
        &mut thr_buffer,
        threshold,
        255.0,
        imgproc::THRESH_BINARY,
    )
    .unwrap();

    // Find matches, each pixel represents the template similarity from 0 (worst) to 1 (best)
    let mut match_result = Mat::default();
    let mask = cv::no_array().unwrap();
    imgproc::match_template(
        &thr_buffer,
        &buffer_template,
        &mut match_result,
        imgproc::TemplateMatchModes::TM_CCOEFF_NORMED as i32,
        &mask,
    )
    .unwrap();
    // Find maximum spots by threshold and count points above threshold
    let mut thr_match_result = Mat::default();
    imgproc::threshold(
        &match_result,
        &mut thr_match_result,
        0.85,
        255.0,
        imgproc::THRESH_BINARY,
    )
    .unwrap();

    let buffer_size: u32 = cv::count_non_zero(&thr_match_result)
        .unwrap()
        .try_into()
        .map_err(|e: TryFromIntError| e.to_string())?;
    Ok(buffer_size)
}

struct CellScanInfo {
    rows: u32,
    cols: u32,
    cells: Vec<cv::Rect>,
}

fn get_contour_rects(img: &Mat, area_threshold: i32) -> Vec<cv::Rect> {
    // Outscribe bounding box around masked cells
    let mut cell_contours = opencv::types::VectorOfVectorOfPoint::new();
    let offset = cv::Point::new(0, 0);
    imgproc::find_contours(
        &img,
        &mut cell_contours,
        imgproc::RETR_LIST,
        imgproc::CHAIN_APPROX_SIMPLE,
        offset,
    )
    .unwrap();

    let mut rects = Vec::new();
    for countour in &cell_contours {
        let rect = imgproc::bounding_rect(&countour).unwrap();
        if rect.area() > area_threshold {
            rects.push(rect);
        }
    }

    rects
}

fn dilate_rect(grid_img: &Mat, ksize: cv::Size, area_threshold: i32) -> Vec<cv::Rect> {
    let anchor = cv::Point::new(-1, -1);
    let border_value = imgproc::morphology_default_border_value().unwrap();
    let kernel = imgproc::get_structuring_element(imgproc::MORPH_RECT, ksize, anchor).unwrap();
    let mut dilate = Mat::default();
    imgproc::dilate(
        &grid_img,
        &mut dilate,
        &kernel,
        anchor,
        1,
        cv::BORDER_ISOLATED,
        border_value,
    )
    .unwrap();

    let mut rects = get_contour_rects(&dilate, area_threshold);
    // transform coordinates from ROI to parent coordinates
    let mut size = cv::Size::new(0, 0);
    let mut offset = cv::Point::new(0, 0);
    grid_img.locate_roi(&mut size, &mut offset).unwrap();
    rects.iter_mut().for_each(|rect| {
        rect.x += offset.x;
        rect.y += offset.y;
    });

    rects
}

fn detect_grid(grey: &Mat) -> Result<CellScanInfo, String> {
    // Blur grid then apply threshold to find cells
    let mut blur = Mat::default();
    let blur_kernel = cv::Size {
        width: 35,
        height: 29,
    };
    imgproc::gaussian_blur(&grey, &mut blur, blur_kernel, 0.0, 0.0, cv::BORDER_DEFAULT).unwrap();
    let mut thr_img = Mat::default();
    let gaussian_threshold = cfg_i32("opencv.detect_grid_threshold");
    imgproc::threshold(
        &blur,
        &mut thr_img,
        gaussian_threshold as f64,
        255.0,
        imgproc::THRESH_BINARY,
    )
    .unwrap();

    // Extract ROI grid max limit area
    let grid_left = cfg_i32("grid.left");
    let grid_right = cfg_i32("grid.right");
    let grid_top = cfg_i32("grid.top");
    let grid_bottom = cfg_i32("grid.bottom");
    let grid_width = grid_right - grid_left;
    let grid_height = grid_bottom - grid_top;
    let cell_min_area = 25 * 25;
    let min_size = cv::Size::new(5, 5);

    let mut roi = cv::Rect::new(grid_left, grid_top, grid_width, grid_height);
    let mut grid_thr_img = Mat::roi(&thr_img, roi).unwrap();
    // let debug_roi_img = Mat::roi(&grey, roi).unwrap();
    // debug_show("roi_img", &debug_roi_img);

    // Dilate horizontally to detect rows
    let dilate_row = 50;
    let kernel_h = cv::Size::new(dilate_row, 1);
    let row_area_threshold = cell_min_area * min_size.width;
    let mut rows = dilate_rect(&grid_thr_img, kernel_h, row_area_threshold);
    // sort rows by y coordinate
    rows.sort_by_key(|row| row.y);
    // debug_contours(&grey, &rows);

    // Adjust top and bottom grid rect if smaller. This avoids extranous data noise during column detection
    roi.y = rows.first().map(|row| row.y).unwrap_or(roi.y);
    let new_grid_bottom_y = rows
        .last()
        .map(|row| row.y + row.height)
        .unwrap_or(roi.y + roi.height);
    roi.height = new_grid_bottom_y - roi.y;
    grid_thr_img = Mat::roi(&thr_img, roi).unwrap();

    // Dilate vertically to detect cols
    let dilate_col = 50;
    let kernel_v = cv::Size::new(1, dilate_col);
    let col_area_threshold = cell_min_area * min_size.height;
    let mut cols = dilate_rect(&grid_thr_img, kernel_v, col_area_threshold);
    // Sort cols by x thr_img coordinate
    cols.sort_by_key(|col| col.x);
    // debug_contours(&grey, &cols);

    // Map rows and cols to cells rectangles
    let mut cells = Vec::new();
    for row in &rows {
        for col in &cols {
            // map to image coordinates
            let cell_rect = cv::Rect::new(col.x, row.y, col.width, row.height);
            cells.push(cell_rect);
        }
    }
    let grid_info = CellScanInfo {
        rows: rows
            .len()
            .try_into()
            .map_err(|e: TryFromIntError| e.to_string())?,
        cols: cols
            .len()
            .try_into()
            .map_err(|e: TryFromIntError| e.to_string())?,
        cells,
    };
    Ok(grid_info)
}

fn detect_daemon_size(grey: &Mat, roi: &cv::Rect) -> Result<CellScanInfo, String> {
    // Blur grid then apply threshold to find cells
    let gaussian_threshold = cfg_i32("opencv.detect_daemon_threshold");
    let mut blur = Mat::default();
    let blur_kernel = cv::Size {
        width: 35,
        height: 29,
    };
    imgproc::gaussian_blur(&grey, &mut blur, blur_kernel, 0.0, 0.0, cv::BORDER_DEFAULT).unwrap();
    let mut thr_img = Mat::default();
    imgproc::threshold(
        &blur,
        &mut thr_img,
        gaussian_threshold as f64,
        255.0,
        imgproc::THRESH_BINARY,
    )
    .unwrap();

    // Get daemon region of interest
    let grid_thr_img = Mat::roi(&thr_img, *roi).unwrap();

    // Dilate vertically to detect cols
    let dilate_col = 50;
    let kernel_v = cv::Size::new(1, dilate_col);
    let cell_min_area = 20 * 20;
    let mut cols = dilate_rect(&grid_thr_img, kernel_v, cell_min_area);
    // Sort cols by x thr_img coordinate
    cols.sort_by_key(|col| col.x);
    // debug_contours(&grey, &cols);

    // Map rows and cols to cells rectangles
    let cells = cols
        .iter()
        .map(|col| cv::Rect::new(col.x, roi.y, col.width, roi.height))
        .collect();
    let grid_info = CellScanInfo {
        rows: 1,
        cols: cols
            .len()
            .try_into()
            .map_err(|e: TryFromIntError| e.to_string())?,
        cells,
    };
    Ok(grid_info)
}

fn scan_daemons(img: &Mat) -> Result<Vec<PuzzleDaemon>, String> {
    let daemon_cfg: DaemonCfg = cfg_get("daemons");
    let rows = daemon_cfg.rows;
    let cell_width = daemon_cfg.cell_width;
    let max_length = daemon_cfg.max_length;
    let max_width = cell_width * max_length;

    let mut daemons = Vec::<PuzzleDaemon>::new();
    for row in &rows {
        let height = row.bottom - row.top;
        let daemon_roi = cv::Rect::new(daemon_cfg.left, row.top, max_width, height);
        let cell_info = detect_daemon_size(img, &daemon_roi).unwrap();
        println!("Daemon size detected: {}", cell_info.cols);
        if cell_info.cols > 0 {
            // Extract sequence cells
            let daemon: PuzzleDaemon = cell_info
                .cells
                .iter()
                .map(|cell| extract_cell(&img, &cell).unwrap())
                .collect();
            // let daemon = cells_txt_result.unwrap();
            daemons.push(daemon);
        }
    }
    Ok(daemons)
}

fn process_grid(grey: &Mat, grid_info: &CellScanInfo) -> Result<Vec<String>, String> {
    // debug_contours(grey, &grid_info.cells);

    let cells_txt: Result<Vec<String>, String> = grid_info
        .cells
        .iter()
        .map(|cell| extract_cell(&grey, &cell))
        .collect();
    cells_txt
}

fn extract_cell(img: &Mat, cell: &cv::Rect) -> Result<String, String> {
    // Helper map to fix most common OCR mistakes
    let correction_map: HashMap<&str, &str> = [("BO", "BD"), ("C", "1C"), ("TA", "7A")]
        .iter()
        .cloned()
        .collect();
    let valid_codes = cfg_str_vec("valid_codes");

    let roi = Mat::roi(img, *cell).unwrap();
    let mut text = recognize_cell(&roi).expect("Failed to recognize grid cell");
    text = correction_map
        .get(text.as_str())
        .map_or(text, |text| (*text).to_owned());

    // Check for invalid code
    if !valid_codes.contains(&text) {
        return Err(format!("An invalid code \"{}\" was recognized", text).to_string());
    }
    Ok(text)
}

// TESTS
#[cfg(test)]
mod tests {
    use super::*;

    static FILE_TEST_5: &str = "assets/images/test_5x5.jpg";
    static FILE_TEST_6: &str = "assets/images/test_6x6.png";
    static FILE_TEST_6_2: &str = "assets/images/test_6x6_2.jpg";
    static FILE_TEST_4_DAEMONS: &str = "assets/images/test_4-daemons.jpg";

    #[test]
    fn test_buffer_detect_7() {
        let test_screen = imread(FILE_TEST_4_DAEMONS, ImreadModes::IMREAD_GRAYSCALE as i32)
            .expect(format!("File {} not found", FILE_TEST_4_DAEMONS).as_str());
        let buffer_size = detect_buffer_size(&test_screen).unwrap();
        assert_eq!(buffer_size, 7);
    }

    #[test]
    fn test_buffer_detect_8() {
        let test_screen = imread(FILE_TEST_6, ImreadModes::IMREAD_GRAYSCALE as i32)
            .expect(format!("File {} not found", FILE_TEST_6).as_str());
        let buffer_size = detect_buffer_size(&test_screen).unwrap();
        assert_eq!(buffer_size, 8);
    }

    #[test]
    fn test_buffer_detect_8_2() {
        let test_screen = imread(FILE_TEST_6_2, ImreadModes::IMREAD_GRAYSCALE as i32)
            .expect(format!("File {} not found", FILE_TEST_6_2).as_str());
        let buffer_size = detect_buffer_size(&test_screen).unwrap();
        assert_eq!(buffer_size, 8);
    }

    #[test]
    fn test_grid_detect_5() {
        let test_screen = imread(FILE_TEST_5, ImreadModes::IMREAD_GRAYSCALE as i32)
            .expect(format!("File {} not found", FILE_TEST_5).as_str());
        let grid_info = detect_grid(&test_screen).unwrap();
        assert_eq!(grid_info.rows, 5);
        assert_eq!(grid_info.cols, 5);
    }

    #[test]
    fn test_grid_detect_6() {
        let test_screen = imread(FILE_TEST_6, ImreadModes::IMREAD_GRAYSCALE as i32)
            .expect(format!("File {} not found", FILE_TEST_6).as_str());
        let grid_info = detect_grid(&test_screen).unwrap();
        assert_eq!(grid_info.rows, 6);
        assert_eq!(grid_info.cols, 6);
    }

    #[test]
    fn test_scan_puzzle_5() {
        let test_screen = imread(FILE_TEST_5, ImreadModes::IMREAD_UNCHANGED as i32)
            .expect(format!("File {} not found", FILE_TEST_5).as_str());
        let puzzle = scan(&test_screen).unwrap();
        #[rustfmt::skip]
        assert_eq!(
            puzzle.grid.cells,
            vec![
                "55","55","1C","55","55",
                "55","E9","BD","1C","BD",
                "E9","1C","1C","1C","55",
                "E9","1C","BD","1C","BD",
                "55","55","BD","55","BD"
            ]
        );
    }

    #[test]
    fn test_scan_puzzle_6() {
        let test_screen = imread(FILE_TEST_6, ImreadModes::IMREAD_UNCHANGED as i32)
            .expect(format!("File {} not found", FILE_TEST_6).as_str());
        let puzzle = scan(&test_screen).unwrap();
        #[rustfmt::skip]
        assert_eq!(
            puzzle.grid.cells,
            vec![
                "E9","1C","55","55","55","1C",
                "55","55","55","7A","BD","BD",
                "BD","E9","E9","55","BD","1C",
                "1C","1C","7A","55","55","7A",
                "7A","7A","55","55","1C","55",
                "E9","E9","1C","BD","55","7A",
            ]
        );
    }

    #[test]
    fn test_scan_daemons() {
        let test_screen = imread(FILE_TEST_4_DAEMONS, ImreadModes::IMREAD_GRAYSCALE as i32)
            .expect(format!("File {} not found", FILE_TEST_4_DAEMONS).as_str());
        let daemons = scan_daemons(&test_screen).unwrap();
        assert_eq!(
            daemons,
            vec![
                vec!["E9", "55"],
                vec!["55", "BD", "E9"],
                vec!["FF", "1C", "BD", "E9"],
                vec!["55", "1C", "FF", "55"]
            ]
        );
    }
}
