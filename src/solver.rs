use std::{collections::HashMap, convert::TryInto};

use crate::types::*;

/// Pair containining a PuzzleMove and its target cell coordinate
type PuzzleMoveWithCoord = (PuzzleMove, CellCoord);

#[derive(Debug, Clone, Copy)]
enum DaemonMatchState {
    Completed,
    Partial(usize),
}

#[derive(Debug, Clone)]
struct SolutionState {
    /// Vector with current Buffer state
    buffer: Vec<String>,
    /// Vector with current sequence of puzzle moves
    moves: PuzzleMoves,
    /// Move count for current step state
    move_count: u32,
    /// Vector of daemon match length, where n-th element is the n-th daemon match length.
    /// The match value can go from 0 (no matches yet) to daemon[n].len() (daemon match completed)
    daemons: Vec<DaemonMatchState>,
    /// Next allowed move type for current state
    next_move_type: PuzzleMoveType,
    /// Used cells map
    used_cells: HashMap<CellCoord, bool>,
}

impl SolutionState {
    /// Create a new initial solution state for a puzzle
    fn new(puzzle: &Puzzle) -> SolutionState {
        let buffer_size = puzzle.buffer_size.try_into().unwrap();
        Self {
            buffer: Vec::with_capacity(buffer_size),
            moves: Vec::with_capacity(buffer_size),
            move_count: 0,
            daemons: vec![DaemonMatchState::Partial(0); puzzle.daemons.len()],
            next_move_type: PuzzleMoveType::SelectColumn,
            used_cells: HashMap::new(),
        }
    }
}

impl std::fmt::Display for SolutionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "\n#{} next: {:?}  {:?}\n",
            self.move_count, self.next_move_type, self.buffer
        ))?;
        f.write_str(&format!("moves: {:?}\n", self.moves))?;
        f.write_str(&format!("daemons: {:?}\n", self.daemons))?;
        f.write_str(&format!("used_cells: {:?}\n", self.used_cells))?;
        Ok(())
    }
}

pub enum SolverSearchMethod {
    Shortest,
    FirstMatch,
}

pub struct BreachSolver<'a> {
    puzzle: &'a Puzzle,
}

impl<'a> BreachSolver<'a> {
    pub fn new(puzzle: &'a Puzzle) -> BreachSolver<'a> {
        BreachSolver { puzzle }
    }
    pub fn solve(&self, method: SolverSearchMethod) -> Option<PuzzleSolution> {
        let mut state = SolutionState::new(&self.puzzle);
        let first_only = matches!(method, SolverSearchMethod::FirstMatch);
        let mut solutions = self.step(&mut state, first_only);
        // Sort solutions by length
        solutions.sort_by_key(|solution| solution.moves.len());
        if let Some(solution) = solutions.get(0) {
            return Some(solution.to_owned());
        }
        None
    }
    pub fn solve_all(&self) -> Vec<PuzzleSolution> {
        let mut state = SolutionState::new(&self.puzzle);
        let mut solutions = self.step(&mut state, false);
        // Sort solutions by length
        solutions.sort_by_key(|solution| solution.moves.len());
        solutions
    }

    fn step(&self, state: &SolutionState, first_only: bool) -> Vec<PuzzleSolution> {
        // Current buffer/move index on which we are iterating in current search step
        let current_move_index: usize = state.move_count.try_into().unwrap();
        let mut solutions: Vec<PuzzleSolution> = Vec::new();

        // Get last move from state, handle implicit first move = Row(0) when state has no moves yet
        let last_move = if current_move_index == 0 {
            PuzzleMove::Row(0)
        } else {
            state.moves[current_move_index - 1]
        };

        // Search available moves on unused and valid cells
        let next_move_type: PuzzleMoveType;
        let is_unused_cell = |(_, cell): &(PuzzleMove, CellCoord)| {
            if let Some(used) = state.used_cells.get(cell) {
                return !*used;
            }
            true
        };
        let available_moves: Vec<PuzzleMoveWithCoord> = match state.next_move_type {
            PuzzleMoveType::SelectColumn => {
                let last_row_index = if let PuzzleMove::Row(index) = last_move {
                    index
                } else {
                    panic!("Expected Row move, but last move was a Column or None move");
                };
                next_move_type = PuzzleMoveType::SelectRow;
                (0..self.puzzle.grid.cols)
                    .map(|col| (PuzzleMove::Column(col), (last_row_index, col)))
                    .filter(is_unused_cell)
                    .collect()
            }
            PuzzleMoveType::SelectRow => {
                let last_col_index = if let PuzzleMove::Column(index) = last_move {
                    index
                } else {
                    panic!("Expected Column move, but last move was a Row or None move");
                };
                next_move_type = PuzzleMoveType::SelectColumn;
                (0..self.puzzle.grid.cols)
                    .map(|row| (PuzzleMove::Row(row), (row, last_col_index)))
                    .filter(is_unused_cell)
                    .collect()
            }
        };

        // Create new state
        let mut new_state = state.to_owned();
        new_state.move_count += 1;
        new_state.next_move_type = next_move_type;

        // Try each available move
        for (new_move, (row, col)) in available_moves {
            let cell_ref = self.puzzle.grid.get_cell(row, col);

            // Update cell usage
            new_state.moves.push(new_move);
            new_state.buffer.push(cell_ref.to_string());
            new_state.used_cells.insert((row, col), true);

            // Update daemon state
            // TODO: (perf) prune if any remaning match len is greater than remaining buffer size
            for (n, daemon) in self.puzzle.daemons.iter().enumerate() {
                let daemon_len = daemon.len();
                let match_state = &mut new_state.daemons[n];
                // We can ignore already completed daemons and check only the remaining ones
                if let DaemonMatchState::Partial(ref mut match_len) = *match_state {
                    // If cell matches daemon cell
                    if *daemon[*match_len] == *cell_ref {
                        *match_len += 1;
                        if *match_len == daemon_len {
                            *match_state = DaemonMatchState::Completed
                        }
                    } else {
                        // If not, reset match
                        *match_len = 0;
                    }
                }
            }

            // Check all daemons for completion
            let all_daemons_completed = new_state
                .daemons
                .iter()
                .all(|daemon| matches!(daemon, DaemonMatchState::Completed));
            if all_daemons_completed {
                // Push valid solution
                solutions.push(PuzzleSolution {
                    moves: new_state.moves.clone(),
                    buffer: new_state.buffer.clone(),
                });
                if first_only {
                    // Stop searching for more solutions on first result
                    break;
                }
            } else {
                if new_state.move_count < self.puzzle.buffer_size {
                    // Only if we can still move, recurse in depth with next move
                    let mut rec_solutions = self.step(&new_state, first_only);
                    solutions.append(&mut rec_solutions);
                }
            }

            // Reset new_state buffer, used_cells and daemons before cycling on next cell
            new_state.daemons.copy_from_slice(&state.daemons);
            new_state.buffer.pop();
            new_state.moves.pop();
            new_state.used_cells.insert((row, col), false);
        }

        solutions
    }

    pub fn to_grid(&self, solution: &PuzzleSolution) -> PuzzleGrid {
        let mut grid = PuzzleGrid::new(self.puzzle.grid.rows, self.puzzle.grid.cols);
        for (i, &(row, col)) in (*solution).to_coords().iter().enumerate() {
            grid.set_cell(row, col, &(i + 1).to_string());
        }
        grid
    }
}

#[cfg(test)]
mod tests {
    use std::iter::FromIterator;

    use super::*;
    use crate::types::PuzzleMove;

    fn to_string_vector(v: Vec<&str>) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    fn moves_to_u32_vec(moves: &PuzzleMoves) -> Vec<u32> {
        moves
            .iter()
            .filter_map(|m| match m {
                &PuzzleMove::Column(i) | &PuzzleMove::Row(i) => Some(i),
                &PuzzleMove::None => None,
            })
            .collect()
    }

    #[test]
    fn test_no_solution() {
        #[rustfmt::skip]
        let test_puzzle_1: Puzzle = Puzzle {
            buffer_size: 8,
            daemons: vec![
                to_string_vector(vec!["BD", "55", "1C"]),
                to_string_vector(vec!["E9", "BD", "1C"]),
                to_string_vector(vec!["1C", "55", "55", "BD"]),
            ],
            grid: PuzzleGrid::from_cells(
                5,
                5,
                vec![
                    "55","55","1C","55","55",
                    "55","E9","BD","1C","BD",
                    "E9","1C","1C","1C","55",
                    "E9","1C","BD","1C","BD",
                    "55","55","BD","55","BD",
                ],
            ),
        };
        let solver = BreachSolver::new(&test_puzzle_1);
        let solution = solver.solve(SolverSearchMethod::FirstMatch);
        assert!(solution.is_none());
    }

    #[test]
    fn test_1_solution() {
        #[rustfmt::skip]
        let test_puzzle_2: Puzzle = Puzzle {
            buffer_size: 7,
            daemons: vec![
                to_string_vector(vec!["1C", "55"]),
                to_string_vector(vec!["55", "55", "55"]),
                to_string_vector(vec!["1C", "1C", "BD"]),
            ],
            grid: PuzzleGrid::from_cells(
                5,
                5,
                vec![
                    "1C","1C","1C","1C","55",
                    "1C","1C","1C","55","55",
                    "E9","55","1C","BD","1C",
                    "55","E9","1C","1C","55",
                    "1C","55","BD","55","1C",
                ],
            ),
        };
        let solver = BreachSolver::new(&test_puzzle_2);

        // solve shortest
        let solution = solver.solve(SolverSearchMethod::Shortest).unwrap();
        assert_eq!(moves_to_u32_vec(&solution.moves), vec![0, 3, 4, 0, 2, 2, 3]);
        assert_eq!(
            solution.buffer,
            vec!["1C", "55", "55", "55", "1C", "1C", "BD"]
        );

        // solve_all
        let solutions = solver.solve_all();
        let str = Vec::from_iter(solutions.iter().map(|s| s.to_string())).join("\n");
        println!("{}", &str);
        assert_eq!(solutions.len(), 18);
        let mapped_solutions: Vec<Vec<u32>> = solutions
            .iter()
            .map(|solution| moves_to_u32_vec(&solution.moves))
            .collect();
        assert_eq!(
            mapped_solutions,
            vec![
                [0, 3, 4, 0, 2, 2, 3],
                [0, 3, 4, 1, 0, 4, 2],
                [0, 3, 4, 1, 2, 2, 3],
                [0, 4, 2, 0, 4, 1, 3],
                [0, 4, 2, 0, 4, 3, 0],
                [0, 4, 2, 1, 3, 4, 1],
                [0, 4, 2, 1, 4, 3, 0],
                [0, 4, 2, 2, 1, 4, 3],
                [0, 4, 2, 3, 4, 1, 3],
                [1, 4, 3, 1, 0, 4, 2],
                [1, 4, 3, 1, 2, 2, 3],
                [2, 2, 3, 0, 4, 1, 3],
                [2, 2, 3, 0, 4, 3, 0],
                [2, 2, 3, 3, 4, 1, 3],
                [3, 1, 4, 0, 0, 4, 2],
                [3, 1, 4, 0, 2, 2, 3],
                [3, 1, 4, 3, 2, 2, 3],
                [3, 4, 1, 2, 4, 4, 2]
            ]
        );
    }

    // #[test]
    #[allow(dead_code)]
    fn test_debug_grid() {
        #[rustfmt::skip]
        let test_puzzle_2: Puzzle = Puzzle {
            buffer_size: 10,
            daemons: vec![to_string_vector(vec![
                "A1", "A2", "B2", "B3", "C3", "C4", "D4", "D5", "E5",
            ])],
            grid: PuzzleGrid::from_cells(
                5,
                5,
                vec![
                    "A1","B1","C1","D1","E1",
                    "A2","B2","C2","D2","E2",
                    "A3","B3","C3","D3","E3",
                    "A4","B4","C4","D4","E4",
                    "A5","B5","C5","D5","E5",
                ],
            ),
        };
        let solver = BreachSolver::new(&test_puzzle_2);
        let solution = solver.solve(SolverSearchMethod::FirstMatch).unwrap();
        println!("{}", solution.to_string());
    }
}
