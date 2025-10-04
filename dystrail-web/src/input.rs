#![allow(clippy::match_same_arms)]

// Centralized numeric keyboard mapping
// Returns Some(0..=9) if the string is a number key; None otherwise
#[must_use]
pub fn numeric_key_to_index(key: &str) -> Option<u8> {
    match key {
        "0" => Some(0),
        "1" => Some(1),
        "2" => Some(2),
        "3" => Some(3),
        "4" => Some(4),
        "5" => Some(5),
        "6" => Some(6),
        "7" => Some(7),
        "8" => Some(8),
        "9" => Some(9),
        _ => None,
    }
}

// Parses KeyboardEvent.code such as "Digit3" or "Numpad5"
#[must_use]
pub fn numeric_code_to_index(code: &str) -> Option<u8> {
    if let Some(last) = code.chars().last()
        && last.is_ascii_digit()
    {
        return numeric_key_to_index(&last.to_string());
    }
    None
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;

    #[test]
    fn key_maps() {
        assert_eq!(numeric_key_to_index("0"), Some(0));
        assert_eq!(numeric_key_to_index("5"), Some(5));
        assert_eq!(numeric_key_to_index("9"), Some(9));
        assert_eq!(numeric_key_to_index("x"), None);
    }

    #[test]
    fn code_maps() {
        assert_eq!(numeric_code_to_index("Digit0"), Some(0));
        assert_eq!(numeric_code_to_index("Numpad5"), Some(5));
        assert_eq!(numeric_code_to_index("KeyA"), None);
    }
}
