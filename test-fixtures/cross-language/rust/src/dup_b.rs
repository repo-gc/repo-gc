// Code-duplication: identical function body to dup_a.rs
pub fn duplicate_function_b(input: &str) -> String {
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

pub fn another_unique_function_b(y: i32) -> i32 {
    y * y + 3 * y + 2
}
