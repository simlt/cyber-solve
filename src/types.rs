pub struct PuzzleGrid {
    pub rows: i32,
    pub cols: i32,
    pub cells: Vec<String>,
}

impl PuzzleGrid {
    pub fn new(rows: i32, cols: i32, cells: Vec<&str>) -> PuzzleGrid {
        PuzzleGrid {
            rows,
            cols,
            cells: cells.iter().map(|s| s.to_string()).collect()
        }
    }

    pub fn row(&self, index: i32) -> Vec<&str> {
        return self.cells[(index * self.cols) as usize..((index + 1) * self.cols) as usize].iter().map(String::as_str).collect();
    }

    pub fn col(&self, index: i32) -> Vec<&str> {
        let mut col = Vec::new();
        for cell in self.cells.iter().skip(index as usize).step_by(self.cols as usize) {
            col.push(cell.as_str());
        }
        return col;
    }

    pub fn get_cell(&self, row: i32, col: i32) -> String {
        return self.cells[(col + row * self.cols) as usize].to_owned();
    }

    pub fn set_cell(&mut self, row: i32, col: i32, value: &str) {
        self.cells[(col + row * self.cols) as usize] = value.to_string();
    }
}

impl std::fmt::Display for PuzzleGrid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let cell_span: usize = 5;
        let col_sep = "|";
        let mut grid_rows: Vec<String> = Vec::new();
        for row in 0..self.rows {
            let row_offset = (row * self.cols) as usize;
            let row_text = col_sep.to_owned()
                + &self.cells[row_offset..(row_offset + self.cols as usize)]
                    .iter()
                    .map(|cell| format!("{:^width$}", cell, width = cell_span))
                    .collect::<Vec<_>>().join(col_sep)
                + col_sep
                + "\n";
            grid_rows.push(row_text);
        }
        let row_sep: &str = &("—".repeat(1 + (cell_span + 1) * self.cols as usize) + "\n");
        let grid_text = row_sep.to_owned() + &grid_rows.join(row_sep) + &row_sep;
        f.write_str(&grid_text)
    }
}

pub type PuzzleDaemon = Vec<String>;

pub fn to_string_vector(v: Vec<&str>) -> Vec<String> {
    v.iter().map(|s| s.to_string()).collect()
}

pub struct Puzzle {
    pub grid: PuzzleGrid,
    pub daemons: Vec<PuzzleDaemon>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid() {
        let grid = PuzzleGrid::new(4, 3,
            vec![
                "0", "1", "2",
                "3", "4", "5",
                "6", "7", "8",
                "9", "10", "11"
            ]
        );

        assert_eq!(grid.row(1), ["3", "4", "5"]);
        assert_eq!(grid.col(1), ["1", "4", "7", "10"]);
        assert_eq!(grid.get_cell(2, 1), "7");
    }
}
