use crate::types::*;

#[derive()]
struct SolutionState {
    buffer: Vec<String>,
    daemons: Vec<u8>,
}

pub(crate) struct BreachSolver<'a> {
    puzzle: &'a Puzzle,
}

impl<'a> BreachSolver<'a> {
    pub(crate) fn new(puzzle: &'a Puzzle) -> BreachSolver<'a> {
        BreachSolver { puzzle }
    }
    pub(crate) fn solve(&self) -> Option<Vec<u8>> {
        let state = SolutionState {
            buffer: vec!["".to_string(); self.puzzle.buffer_size.into()],
            daemons: vec![0; self.puzzle.daemons.len()]
        };

        /* 
        const buffer: ByteCode[] = [];
        const daemonStatus: DaemonStatus = Object.fromEntries(
        Object.entries(this.puzzle.daemons).map(([index, sequence]) => [
            index,
            sequence.length,
        ])
        );
        let solutions = this.step(this.puzzle.grid, daemonStatus, buffer, 0, true);
        if (solutions) {
        solutions = sortBy(solutions, (s) => s.length);
        return solutions[0];
        }
        */
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
