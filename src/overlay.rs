use crate::scanner::debug_show;
use crate::types::PuzzleGrid;
use crate::utils::lerp_i;
use crate::utils::Color;
use opencv::core as cv;
use opencv::imgproc;
use opencv::prelude::*;

pub(crate) fn show(grid: &PuzzleGrid, (x, y): (i32, i32), (height, width): (i32, i32)) -> () {
    // let bg_color = Color::rgba(255,255,255,0).to_bgra();
    let bg_color = Color::rgba(0, 0, 0, 0).to_bgra();
    let mut img = Mat::new_rows_cols_with_default(height, width, cv::CV_8UC4, bg_color).unwrap();
    draw_grid(&mut img, &grid);
    // save_image();
    // load_overlay_image();
}

fn draw_grid(img: &mut Mat, grid: &PuzzleGrid) -> () {
    let rows = grid.rows;
    let cols = grid.cols;
    let height = img.rows();
    let width = img.cols();
    let cyber_yellow = Color::rgba(0xcf, 0xed, 0x56, 0xff).to_bgra(); // #cfed56
    let thickness = 3;
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
    let font_face = imgproc::FONT_HERSHEY_DUPLEX;
    let font_scale = 2.5f64;
    let thickness = 1;
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
    debug_show("overlay grid cells", img);
}

#[cfg(test)]
mod tests {
    use crate::overlay::*;
    use crate::types::PuzzleGrid;

    #[test]
    fn test_overlay() {
        let size = 5;
        let cells: Vec<String> = (0..size * size).map(|i| i.to_string().to_owned()).collect();
        let grid = PuzzleGrid::from_cells(size, size, cells);

        show(&grid, (100, 100), (600, 600));
    }
}
