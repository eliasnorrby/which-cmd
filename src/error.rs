use thiserror::Error;

/// Custom error type for which-cmd operations
#[derive(Error, Debug)]
pub enum WhichCmdError {
    /// Configuration file was not found
    #[error("Configuration file not found at {path}")]
    ConfigNotFound { path: String },

    /// Duplicate keys found in configuration
    #[error("Conflicting keys found: {0}")]
    ConflictingKeys(String),

    /// Invalid YAML in configuration file
    #[error("Failed to parse configuration: {0}")]
    ConfigParse(#[from] serde_yaml::Error),

    /// IO error reading configuration
    #[error("Failed to read configuration: {0}")]
    ConfigIo(#[from] std::io::Error),

    /// XDG directory error
    #[error("Failed to access XDG directories: {0}")]
    Xdg(#[from] xdg::BaseDirectoriesError),

    /// Terminal operation failed
    #[error("Terminal error: {0}")]
    Terminal(String),

    /// No selection was made when one was required
    #[error("No selection made")]
    #[allow(dead_code)]
    NoSelection,

    /// User cancelled the operation
    #[error("Operation cancelled by user")]
    #[allow(dead_code)]
    Cancelled,
}

/// Convenience type alias for Results using WhichCmdError
pub type Result<T> = std::result::Result<T, WhichCmdError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_not_found_error_display() {
        let error = WhichCmdError::ConfigNotFound {
            path: "/home/user/.config/which-cmd/commands.yml".to_string(),
        };

        let display = format!("{}", error);
        assert!(display.contains("Configuration file not found"));
        assert!(display.contains("/home/user/.config/which-cmd/commands.yml"));
    }

    #[test]
    fn test_conflicting_keys_error_display() {
        let error = WhichCmdError::ConflictingKeys("gs".to_string());

        let display = format!("{}", error);
        assert!(display.contains("Conflicting keys found"));
        assert!(display.contains("gs"));
    }

    #[test]
    fn test_terminal_error_display() {
        let error = WhichCmdError::Terminal("Failed to read input".to_string());

        let display = format!("{}", error);
        assert!(display.contains("Terminal error"));
        assert!(display.contains("Failed to read input"));
    }

    #[test]
    fn test_io_error_conversion() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let which_cmd_error: WhichCmdError = io_error.into();

        let display = format!("{}", which_cmd_error);
        assert!(display.contains("Failed to read configuration"));
    }

    #[test]
    fn test_yaml_error_conversion() {
        let yaml = "invalid: yaml: content:";
        let yaml_error = serde_yaml::from_str::<serde_yaml::Value>(yaml).unwrap_err();
        let which_cmd_error: WhichCmdError = yaml_error.into();

        let display = format!("{}", which_cmd_error);
        assert!(display.contains("Failed to parse configuration"));
    }

    #[test]
    fn test_xdg_error_from_trait() {
        // Test that xdg::BaseDirectoriesError can be converted to WhichCmdError
        // using the #[from] attribute
        fn trigger_xdg_error() -> Result<()> {
            // This will return an xdg error which gets auto-converted via From trait
            let _xdg_dirs = xdg::BaseDirectories::with_prefix("which-cmd")?;
            Ok(())
        }

        // The function should succeed in this case
        let result = trigger_xdg_error();
        assert!(result.is_ok());

        // Just verify we can match on the Xdg variant
        // (we can't easily construct an XDG error, but we can verify the pattern works)
        let test_error = WhichCmdError::ConfigNotFound {
            path: "test".to_string(),
        };
        match test_error {
            WhichCmdError::Xdg(_) => panic!("Should not be Xdg"),
            WhichCmdError::ConfigNotFound { .. } => {
                // This is expected
            }
            _ => panic!("Expected ConfigNotFound variant"),
        }
    }

    #[test]
    fn test_error_is_send_and_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}

        assert_send::<WhichCmdError>();
        assert_sync::<WhichCmdError>();
    }

    #[test]
    fn test_result_type_alias() {
        fn returns_result() -> Result<i32> {
            Ok(42)
        }

        let result = returns_result();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_result_type_alias_with_error() {
        fn returns_error() -> Result<i32> {
            Err(WhichCmdError::ConflictingKeys("test".to_string()))
        }

        let result = returns_error();
        assert!(result.is_err());
    }
}
