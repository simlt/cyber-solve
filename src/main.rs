use core::time;
use std::thread;

mod configuration;
mod ocr;
mod overlay;
mod scanner;
mod screenshot;
mod solver;
mod types;
mod utils;

fn main() {
    let mut overlay = overlay::Overlay::new();
    let five_secs = time::Duration::from_secs(5);
    let fifteen_secs = time::Duration::from_secs(15);

    loop {
        match scanner::capture_and_scan() {
            Ok(puzzle) => {
                let solver = solver::BreachSolver::new(&puzzle);
                if let Some(solution) = solver.solve(solver::SolverSearchMethod::Shortest) {
                    let grid = solver.to_grid(&solution);
                    println!("{}", grid.to_string());
                    overlay.show(&grid);
                } else {
                    println!("No solution found");
                }
                thread::sleep(fifteen_secs);
            }
            Err(msg) => {
                println!("Scan failed: {}", &msg);
                overlay.hide();
                thread::sleep(five_secs);
            }
        }
    }
}
