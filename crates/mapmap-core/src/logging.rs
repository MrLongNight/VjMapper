//! Centralized Logging Configuration
//!
//! Provides file-based logging with rotation and configurable log levels.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Logging configuration for the application
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LogConfig {
    /// Log level (trace, debug, info, warn, error)
    pub level: LogLevel,
    /// Directory for log files
    pub log_directory: PathBuf,
    /// Maximum number of log files to retain
    pub max_log_files: usize,
    /// Enable console output in addition to file
    pub console_output: bool,
}

/// Log level enumeration
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum LogLevel {
    /// Most verbose - includes all trace statements
    Trace,
    /// Debug information
    Debug,
    /// General information (default)
    #[default]
    Info,
    /// Warnings
    Warn,
    /// Errors only
    Error,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::Info,
            log_directory: PathBuf::from("logs"),
            max_log_files: 10,
            console_output: true,
        }
    }
}

impl LogLevel {
    /// Get string representation of log level
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Trace => "trace",
            LogLevel::Debug => "debug",
            LogLevel::Info => "info",
            LogLevel::Warn => "warn",
            LogLevel::Error => "error",
        }
    }

    /// Get all log levels for UI selection
    pub fn all() -> Vec<LogLevel> {
        vec![
            LogLevel::Trace,
            LogLevel::Debug,
            LogLevel::Info,
            LogLevel::Warn,
            LogLevel::Error,
        ]
    }

    /// Get display name for UI
    pub fn display_name(&self) -> &'static str {
        match self {
            LogLevel::Trace => "Trace (Verbose)",
            LogLevel::Debug => "Debug",
            LogLevel::Info => "Info",
            LogLevel::Warn => "Warning",
            LogLevel::Error => "Error Only",
        }
    }
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
