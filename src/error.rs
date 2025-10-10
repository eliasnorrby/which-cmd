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
    NoSelection,

    /// User cancelled the operation
    #[error("Operation cancelled by user")]
    Cancelled,
}

/// Convenience type alias for Results using WhichCmdError
pub type Result<T> = std::result::Result<T, WhichCmdError>;
