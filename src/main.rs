mod scanner;
mod configuration;

fn main() {
    scanner::capture_and_scan().unwrap();
}
