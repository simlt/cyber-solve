mod configuration;
mod ocr;
mod overlay;
mod scanner;
mod screenshot;
mod solver;
mod types;
mod utils;

fn main() {
    match scanner::capture_and_scan() {
        Ok(puzzle) => {
            let solver = solver::BreachSolver::new(&puzzle);
            if let Some(solution) = solver.solve(solver::SolverSearchMethod::Shortest) {
                let grid = solver.to_grid(&solution);
                println!("{}", grid.to_string());
                overlay::show(&grid, (100, 100), (600, 400));
            } else {
                println!("No solution found");
            }
        }
        Err(msg) => {
            println!("Scan failed: {}", &msg);
        }
    }
}
