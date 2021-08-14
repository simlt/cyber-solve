use std::env::current_dir;
use std::io::Error;
use std::process::Child;
use std::process::Command;
use std::time::SystemTime;

use crate::types::PuzzleGrid;
use crate::utils::lerp_i;
use crate::utils::Color;
use opencv::core as cv;
use opencv::imgcodecs::imwrite;
use opencv::imgproc;
use opencv::prelude::*;
use tempfile;

pub(crate) struct Overlay {
    pub child_process: Option<Child>,
    tmp_dir: tempfile::TempDir,
}

impl Drop for Overlay {
    fn drop(&mut self) {
        // Make sure to kill child process on exit, ignore errors
        let _ = self.kill_overlay_process();
    }
}

impl Overlay {
    pub(crate) fn new() -> Overlay {
        let tmp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        Overlay {
            tmp_dir,
            child_process: None,
        }
    }

    pub(crate) fn show(&mut self, grid: &PuzzleGrid) -> () {
        let x = 852;
        let y = 715;
        let image_width = 500;
        let image_height = 500;
        let overlay_width = 215;
        let overlay_height = 215;

        // let bg_color = Color::rgba(255,255,255,0).to_bgra();
        let bg_color = Color::rgba(0, 0, 0, 0).to_bgra();
        let mut img =
            Mat::new_rows_cols_with_default(image_height, image_width, cv::CV_8UC4, bg_color)
                .unwrap();
        draw_grid(&mut img, &grid);
        let path = self.save_image(&img);
        self.load_overlay_image(&path, x, y, overlay_width, overlay_height);
    }

    pub(crate) fn hide(&mut self) -> () {
        let _ = self.kill_overlay_process();
    }

    fn load_overlay_image(&mut self, path: &str, x: i32, y: i32, width: i32, height: i32) -> () {
        let args = [
            path,
            &x.to_string(),
            &y.to_string(),
            &width.to_string(),
            &height.to_string(),
        ];
        let parent_dir = current_dir().unwrap();
        let assets_bin = parent_dir.join("assets/bin").canonicalize().unwrap();
        let overlay_exe_path = assets_bin.join("overlay.exe");

        // println!("{}", assets_bin.display());
        let process = Command::new(overlay_exe_path)
            .args(&args)
            .current_dir(assets_bin)
            .spawn()
            .expect("Failed to start overlay process");

        println!(
            "Started ovelay child process pid: {} with image: {}",
            process.id(),
            path
        );

        // Kill old child process
        if let Err(err) = self.kill_overlay_process() {
            println!("Could not kill child process. {}", err.to_string());
        }

        // Update current child process
        self.child_process = Some(process);
    }

    fn kill_overlay_process(&mut self) -> Result<(), Error> {
        if let Some(process) = &mut self.child_process {
            process.kill()?;
        }
        Ok(())
    }

    fn save_image(&self, img: &Mat) -> String {
        // Save to png image to support alpha channel
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();
        let path = self
            .tmp_dir
            .path()
            .join(format!("cybersolve-overlay-{}.png", now.as_secs()));
        let path_str = path.to_str().unwrap();
        let imwrite_flags = cv::Vector::new();
        imwrite(&path_str, img, &imwrite_flags).expect("Failed to write image file");
        println!("Overlay image saved at \"{}\"", &path_str);

        path_str.to_owned()
    }
}

fn draw_grid(img: &mut Mat, grid: &PuzzleGrid) -> () {
    let rows = grid.rows;
    let cols = grid.cols;
    let height = img.rows();
    let width = img.cols();
    let cyber_yellow = Color::rgba(0xcf, 0xed, 0x56, 0xff).to_bgra(); // #cfed56
    let thickness = 2;
    let line_type = imgproc::LineTypes::LINE_8 as i32;

    // Grid size
    let margin = 0;
    let x_left = margin;
    let x_right = width - margin;
    let y_top = margin;
    let y_bottom = height - margin;
    let grid_width = x_right - x_left;
    let grid_height = y_bottom - y_top;
    let grid_top_left = cv::Point::new(x_left, y_top);

    // Draw GRID lines
    // Vertical lines
    for col in 0..=cols {
        let x = lerp_i(0, grid_width, col as f64 / cols as f64);
        let top = grid_top_left + cv::Point::new(x, 0);
        let bottom = top + cv::Point::new(0, grid_height);
        imgproc::line(img, top, bottom, cyber_yellow, thickness, line_type, 0).unwrap();
    }
    // Horizontal lines
    for row in 0..=rows {
        let y = lerp_i(0, grid_height, row as f64 / rows as f64);
        let left = grid_top_left + cv::Point::new(0, y);
        let right = left + cv::Point::new(grid_width, 0);
        imgproc::line(img, left, right, cyber_yellow, thickness, line_type, 0).unwrap();
    }
    // debug_show("overlay grid lines", img);

    // Draw GRID cells
    let cell_width = ((grid_width) as f64 / (cols) as f64).round() as i32;
    let cell_height = ((grid_height) as f64 / (rows) as f64).round() as i32;
    let first_cell_origin = grid_top_left + cv::Point::new(cell_width, cell_height) / 2;
    // TODO: use better font
    // Original font should be Eurocine Narrow. Rajdhani or Noto Sans are a similar alternative
    let font_face = imgproc::FONT_HERSHEY_SIMPLEX;
    let font_scale = 2.0f64;
    let thickness = 2;
    let text_line_type = imgproc::LineTypes::LINE_AA as i32;
    for row in 0..rows {
        let y_offset = lerp_i(0, grid_height - cell_height, row as f64 / (rows - 1) as f64);
        for col in 0..cols {
            let cell = grid.get_cell(row, col);
            let x_offset = lerp_i(0, grid_width - cell_width, col as f64 / (cols - 1) as f64);
            let mut base_line = 0;
            let text_size =
                imgproc::get_text_size(cell, font_face, font_scale, thickness, &mut base_line)
                    .unwrap();
            // Position text from first cell center, add grid offset, then offset by half the text size wtr cell center
            let text_origin = first_cell_origin
                + cv::Point::new(x_offset, y_offset)
                + cv::Point::new(-text_size.width, text_size.height) / 2;
            imgproc::put_text(
                img,
                cell,
                text_origin,
                font_face,
                font_scale,
                cyber_yellow,
                thickness,
                text_line_type,
                false,
            )
            .unwrap();
        }
    }
    // debug_show("overlay grid cells", img);
}

#[cfg(test)]
mod tests {
    use crate::overlay::*;
    use crate::types::PuzzleGrid;

    #[test]
    fn test_overlay() {
        // Make 1,2,...,25 test grid
        let size = 5;
        let cells: Vec<String> = (0..size * size).map(|i| i.to_string().to_owned()).collect();
        let grid = PuzzleGrid::from_cells(size, size, cells);

        let mut overlay = Overlay::new();
        overlay.show(&grid);
    }
}
