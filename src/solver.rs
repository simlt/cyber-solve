use std::collections::HashMap;

use crate::types::*;


/// row, col index pair for cell grid coordinate
type CellCoord = (u32, u32);
/// Pair containining a PuzzleMove and its target cell coordinate
type PuzzleMoveWithCoord = (PuzzleMove, CellCoord);

#[derive(Clone, Copy)]
enum DaemonMatchState {
    Completed,
    Partial(usize),
}


#[derive()]
struct SolutionState {
    /// Vector with current Buffer state
    buffer: Vec<String>,
    /// Vector with current row-col step state
    moves: Vec<PuzzleMove>,
    /// Move count for current step state
    move_count: usize,
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
        Self {
            buffer: vec!["".to_string(); puzzle.buffer_size.into()],
            moves: vec![PuzzleMove::None; puzzle.buffer_size.into()],
            move_count: 0,
            daemons: vec![DaemonMatchState::Partial(0); puzzle.daemons.len()],
            next_move_type: PuzzleMoveType::SelectColumn,
            used_cells: HashMap::new(),
        }
    }
}

pub(crate) struct BreachSolver<'a> {
    puzzle: &'a Puzzle,
}

impl<'a> BreachSolver<'a> {
    pub(crate) fn new(puzzle: &'a Puzzle) -> BreachSolver<'a> {
        BreachSolver { puzzle }
    }
    pub(crate) fn solve(&self) -> Option<Vec<u32>> {
        let state = SolutionState::new(&self.puzzle);
        let solutions = self.step(&mut state, false);
        /*
        let solutions = this.step(this.puzzle.grid, daemonStatus, buffer, 0, true);
        if (solutions) {
        solutions = sortBy(solutions, (s) => s.length);
        return solutions[0];
        }
        */
        None
    }

    fn step(&self, state: &mut SolutionState, first_only: bool) -> Vec<Vec<PuzzleMove>> {
        let mut solutions: Vec<Vec<PuzzleMove>> = Vec::new();

        // Get last move from state, handle implicit  first move = Row(0) when state has no moves yet
        let last_move = if state.move_count == 0 {
            PuzzleMove::Row(0)
        } else {
            state.moves[state.move_count - 1]
        };

        // Search available moves on unused and valid cells
        let is_unused_cell = |(_, cell): &(PuzzleMove, CellCoord)| *state.used_cells.get(cell).unwrap_or(&false);
        let available_moves: Vec<PuzzleMoveWithCoord> = match state.next_move_type {
            PuzzleMoveType::SelectColumn => {
                let last_row_index = if let PuzzleMove::Row(index) = last_move {
                    index
                } else {
                    panic!("Expected Row move, but last move was a Column or None move");
                };
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
                (0..self.puzzle.grid.cols)
                    .map(|row| (PuzzleMove::Row(row), (row, last_col_index)))
                    .filter(is_unused_cell)
                    .collect()
            }
        };

        // Try each available move
        state.move_count += 1;
        for (new_move, (row, col)) in available_moves {
            let cell_ref = self.puzzle.grid.get_cell(row, col);

            // Update daemon state
            for (n, daemon) in self.puzzle.daemons.iter().enumerate() {
                let daemon_len = daemon.len();
                let match_state = &mut state.daemons[n];
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

            // Update cell usage
            state.moves[state.move_count - 1] = new_move;
            state.buffer[state.move_count - 1] = *cell_ref;
            state.used_cells[&(row, col)] = true;

            // Check all daemons for completion
            let all_daemons_completed = state.daemons.iter().all(|daemon| matches!(daemon, DaemonMatchState::Completed));
            if all_daemons_completed {
                // Push valid solution
                solutions.push(state.moves);
                if first_only {
                    // Stop searching for more solutions on first result
                    break;
                }
            } else {
                if state.move_count < self.puzzle.buffer_size {
                    // Only if we can still move, recurse in depth with next move
                    let rec_solutions = self.step(state, first_only);
                    solutions.append(&mut rec_solutions);
                }
            }

            // Reset used_cell before cycling on next cell
            state.used_cells[&(row, col)] = false;
        }

        solutions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_solution() {
        let test_puzzle_1: Puzzle = Puzzle {
            buffer_size: 8,
            daemons: vec![
                to_string_vector(vec!["BD", "55", "1C"]),
                to_string_vector(vec!["E9", "BD", "1C"]),
                to_string_vector(vec!["1C", "55", "55", "BD"]),
            ],
            grid: PuzzleGrid::new(
                5,
                5,
                vec![
                    "55","55","1C","55","55",
                    "55","E9","BD","1C","BD",
                    "E9","1C","1C","1C","55",
                    "E9","1C","BD","1C","BD",
                    "55","55","BD","55","BD",
                ],
            )
        };
        let solver = BreachSolver::new(&test_puzzle_1);
        let solutions = solver.solve();
        assert_eq!(solutions, None);
    }
    
    #[test]
    fn test_1_solution() {
        let test_puzzle_2: Puzzle = Puzzle {
            buffer_size: 7,
            daemons: vec![
                to_string_vector(vec!["1C","55"]),
                to_string_vector(vec!["55","55","55"]),
                to_string_vector(vec!["1C","1C","BD"]),
            ],
            grid: PuzzleGrid::new(
                5,
                5,
                vec![
                    "1C","1C","1C","1C","55",
                    "1C","1C","1C","55","55",
                    "E9","55","1C","BD","1C",
                    "55","E9","1C","1C","55",
                    "1C","55","BD","55","1C",
                ],
            )
        };
        let solver = BreachSolver::new(&test_puzzle_2);
        let solution = solver.solve();
        assert_eq!(solution, Some(vec![0, 3, 4, 0, 2, 2, 3]));
        // TODO: get buffer from solution
        // assert_eq!(solution_buffer, ["1C","55","55","55","1C","1C","BD"]);
    }
}
