// Error-swallow: ignoring errors with if let Err(_)
use std::fs::File;
use std::io::Read;

pub fn read_file_silent(path: &str) {
    if let Err(_) = File::open(path) {
        // silently ignore
    }
}

pub fn parse_number_silent(input: &str) -> i32 {
    if let Err(_) = input.parse::<i32>() {
        return 0;
    }
    42
}

pub fn fetch_data_silent(url: &str) {
    if let Err(_) = reqwest::blocking::get(url) {
        // do nothing
    }
}
