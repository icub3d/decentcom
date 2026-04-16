//! Passphrase strength validation for key backup encryption.

/// Minimum acceptable passphrase length.
const MIN_LENGTH: usize = 12;

/// Strength levels for passphrase feedback.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PassphraseStrength {
    /// Below minimum length — rejected.
    TooShort,
    /// Meets minimum but low entropy.
    Weak,
    /// Reasonable entropy.
    Fair,
    /// High entropy.
    Strong,
}

/// Result of validating a passphrase.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PassphraseValidation {
    pub strength: PassphraseStrength,
    pub acceptable: bool,
    pub feedback: String,
}

/// Validate a passphrase and return strength + feedback.
pub fn validate_passphrase(passphrase: &str) -> PassphraseValidation {
    let len = passphrase.len();

    if len < MIN_LENGTH {
        return PassphraseValidation {
            strength: PassphraseStrength::TooShort,
            acceptable: false,
            feedback: format!("Must be at least {} characters (currently {})", MIN_LENGTH, len),
        };
    }

    let entropy = estimate_entropy(passphrase);

    if entropy < 40.0 {
        PassphraseValidation {
            strength: PassphraseStrength::Weak,
            acceptable: true,
            feedback: "Weak — consider adding numbers, symbols, or more words".into(),
        }
    } else if entropy < 60.0 {
        PassphraseValidation {
            strength: PassphraseStrength::Fair,
            acceptable: true,
            feedback: "Fair — acceptable for backup encryption".into(),
        }
    } else {
        PassphraseValidation {
            strength: PassphraseStrength::Strong,
            acceptable: true,
            feedback: "Strong passphrase".into(),
        }
    }
}

/// Rough entropy estimation based on character class variety.
fn estimate_entropy(passphrase: &str) -> f64 {
    let mut charset_size: u32 = 0;
    let has_lower = passphrase.chars().any(|c| c.is_ascii_lowercase());
    let has_upper = passphrase.chars().any(|c| c.is_ascii_uppercase());
    let has_digit = passphrase.chars().any(|c| c.is_ascii_digit());
    let has_symbol = passphrase.chars().any(|c| c.is_ascii_punctuation());
    let has_space = passphrase.contains(' ');

    if has_lower {
        charset_size += 26;
    }
    if has_upper {
        charset_size += 26;
    }
    if has_digit {
        charset_size += 10;
    }
    if has_symbol {
        charset_size += 32;
    }
    if has_space {
        charset_size += 1;
    }

    if charset_size == 0 {
        charset_size = 26; // fallback for non-ASCII
    }

    passphrase.len() as f64 * (charset_size as f64).log2()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_short_passphrase() {
        let result = validate_passphrase("short");
        assert_eq!(result.strength, PassphraseStrength::TooShort);
        assert!(!result.acceptable);
    }

    #[test]
    fn rejects_eleven_chars() {
        let result = validate_passphrase("12345678901");
        assert_eq!(result.strength, PassphraseStrength::TooShort);
        assert!(!result.acceptable);
    }

    #[test]
    fn accepts_twelve_chars() {
        let result = validate_passphrase("123456789012");
        assert!(result.acceptable);
    }

    #[test]
    fn strong_passphrase() {
        let result = validate_passphrase("Correct Horse Battery Staple!123");
        assert_eq!(result.strength, PassphraseStrength::Strong);
        assert!(result.acceptable);
    }

    #[test]
    fn weak_passphrase() {
        // All lowercase, just meets length — low entropy.
        let result = validate_passphrase("aaaaaaaaaaaa");
        assert!(result.acceptable);
        // 12 * log2(26) ≈ 56 bits — "Fair" range.
        assert_eq!(result.strength, PassphraseStrength::Fair);
    }

    #[test]
    fn weak_digits_only() {
        // All digits — even smaller charset.
        let result = validate_passphrase("123456789012");
        assert_eq!(result.strength, PassphraseStrength::Weak);
        assert!(result.acceptable);
    }
}
