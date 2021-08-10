mod configuration;
mod ocr;
mod scanner;
mod solver;
mod types;

fn main() {
    let puzzle = scanner::capture_and_scan().unwrap();
    let solver = solver::BreachSolver::new(&puzzle);
    solver.solve();
}
