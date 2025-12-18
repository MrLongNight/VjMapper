pub mod protocol;
pub mod server;

use std::path::PathBuf;

pub use protocol::*;
pub use server::McpServer;

// Re-export for convenience
pub use anyhow::Result;

/// Actions internally triggered by the MCP Server to be handled by the main application.
#[derive(Debug, Clone)]
pub enum McpAction {
    /// Save the project.
    SaveProject(PathBuf),
    /// Load a project.
    LoadProject(PathBuf),
    /// Add a new layer.
    AddLayer,
    /// Remove a layer by ID.
    RemoveLayer(u64),
    /// Trigger a cue by ID.
    TriggerCue(u64),
    /// Go to the next cue.
    NextCue,
    /// Go to the previous cue.
    PrevCue,
}
