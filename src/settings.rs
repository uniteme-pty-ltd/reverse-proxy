use std::env;

// Get a setting from the environment
pub fn setting(key: &str) -> Option<String> {
    match env::var(key) {
        Ok(val) => match val.as_str() {
            "" => None,
            _ => Some(val),
        },
        Err(_) => None,
    }
}
