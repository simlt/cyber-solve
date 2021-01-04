mod configuration;
mod ocr;
mod scanner;
mod types;

fn main() {
    scanner::capture_and_scan().unwrap();
}
