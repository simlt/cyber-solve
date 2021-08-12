mod configuration;
mod ocr;
mod scanner;
mod screenshot;
mod solver;
mod types;

fn main() {
    match scanner::capture_and_scan() {
        Ok(puzzle) => {
            let solver = solver::BreachSolver::new(&puzzle);
            if let Some(solution) = solver.solve(solver::SolverSearchMethod::Shortest) {
                let grid = solver.to_grid(&solution);
                println!("{}", grid.to_string())
            } else {
                println!("No solution found");
            }
        },
        Err(msg) => {
            println!("Scan failed: {}", &msg);
        }
    }
}
