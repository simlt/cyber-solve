pub struct PuzzleGrid {
    pub rows: i32,
    pub cols: i32,
    pub cells: Vec<String>,
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

pub struct PuzzleDaemon {}

pub struct Puzzle {
    pub grid: PuzzleGrid,
    pub daemons: Vec<PuzzleDaemon>,
}
