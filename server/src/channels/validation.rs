use std::fmt;

const MAX_CHANNEL_NAME_LEN: usize = 100;

#[derive(Debug, PartialEq, Eq)]
pub enum ChannelNameError {
    Empty,
    TooLong,
    InvalidChars,
}

impl fmt::Display for ChannelNameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChannelNameError::Empty => write!(f, "channel name must not be empty"),
            ChannelNameError::TooLong => write!(
                f,
                "channel name must be at most {MAX_CHANNEL_NAME_LEN} characters"
            ),
            ChannelNameError::InvalidChars => write!(
                f,
                "channel name may only contain lowercase letters, digits, and hyphens"
            ),
        }
    }
}

/// Returns `Ok(())` if `name` is a valid channel name, else `Err(ChannelNameError)`.
pub fn validate_channel_name(name: &str) -> Result<(), ChannelNameError> {
    if name.is_empty() {
        return Err(ChannelNameError::Empty);
    }
    if name.len() > MAX_CHANNEL_NAME_LEN {
        return Err(ChannelNameError::TooLong);
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        return Err(ChannelNameError::InvalidChars);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_names() {
        for name in &["general", "off-topic", "dev-123", "a", "z-9"] {
            assert!(
                validate_channel_name(name).is_ok(),
                "{name} should be valid"
            );
        }
    }

    #[test]
    fn empty_name_rejected() {
        assert_eq!(validate_channel_name(""), Err(ChannelNameError::Empty));
    }

    #[test]
    fn too_long_name_rejected() {
        let long = "a".repeat(101);
        assert_eq!(
            validate_channel_name(&long),
            Err(ChannelNameError::TooLong)
        );
    }

    #[test]
    fn invalid_chars_rejected() {
        for name in &["General", "off topic", "dev_123", "hello!", "#chat"] {
            assert_eq!(
                validate_channel_name(name),
                Err(ChannelNameError::InvalidChars),
                "{name} should be invalid"
            );
        }
    }
}
