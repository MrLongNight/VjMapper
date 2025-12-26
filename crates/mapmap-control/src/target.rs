//! Control target abstraction
//!
//! This module provides a unified abstraction for all controllable parameters in MapFlow.

// Re-export from mapmap-core to avoid duplication and cyclic dependencies
pub use mapmap_core::assignment::{ControlTarget, ControlValue, EdgeSide};
