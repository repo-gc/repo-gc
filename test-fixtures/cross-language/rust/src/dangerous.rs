// Dangerous pattern: unsafe blocks and unwrap calls
pub fn dangerous_function() {
    let val = "not a number".parse::<i32>().unwrap();
    let data = vec![1, 2, 3];
    let _first = data.get(10).unwrap();
    let _json: serde_json::Value = serde_json::from_str("invalid").unwrap();
}

pub unsafe fn unsafe_helper(ptr: *const i32) -> i32 {
    *ptr
}

pub fn risky_network() {
    let _resp = reqwest::blocking::get("https://example.com").unwrap();
}
