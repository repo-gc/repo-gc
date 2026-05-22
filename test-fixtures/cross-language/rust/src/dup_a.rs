// Code-duplication: identical function body to dup_b.rs
pub fn duplicate_function_a(input: &str) -> String {
    let mut result = String::new();
    for ch in input.chars() {
        if ch.is_alphabetic() {
            result.push(ch.to_ascii_uppercase());
        } else if ch.is_numeric() {
            result.push(ch);
        } else {
            result.push('_');
        }
    }
    result
}

pub fn another_unique_function_a(x: i32) -> i32 {
    x * x + 2 * x + 1
}
