//! Random text from the system random generator. Used for the access token
//! and the session ids, so it must never be a weak generator.

/// `bytes` random bytes written as lowercase hex (2 characters per byte).
pub fn random_hex(bytes: usize) -> String {
    let mut buffer = vec![0u8; bytes];
    // The system generator can only fail on a broken system. Without good
    // randomness there is no safe token, so stopping is the safe choice.
    getrandom::fill(&mut buffer).expect("the system random generator failed");
    buffer.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // Checks the length and the alphabet, and that two calls differ (the
    // chance of a clash for 16 bytes is zero in practice).
    #[test]
    fn random_hex_has_the_right_shape() {
        let a = random_hex(16);
        assert_eq!(a.len(), 32);
        assert!(a.chars().all(|c| c.is_ascii_hexdigit()));
        assert_ne!(a, random_hex(16));
    }
}
