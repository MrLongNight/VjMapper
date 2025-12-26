//! Control Assignment System
//!
//! This module defines the data structures for assigning control sources
//! (like MIDI CCs, LFOs, or audio analysis) to control targets (like layer
//! opacity, effect parameters, etc.).

use serde::{Deserialize, Serialize};

/// A single control assignment.
///
/// This struct links a control source to a control target using unique `u64` IDs
/// to avoid brittle dependencies on names.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Assignment {
    /// Unique ID of this assignment.
    pub id: u64,
    /// ID of the control source (e.g., a specific MIDI CC on a specific device).
    pub source_id: u64,
    /// ID of the target module (e.g., a Layer, an Effect).
    pub target_module_id: u64,
    /// ID of the specific part within the module (e.g., the Transform part of a Layer).
    pub target_part_id: u64,
    /// The named parameter within the target part (e.g., "opacity", "scale_x").
    pub target_param_name: String,
    /// A user-friendly name for this assignment.
    pub name: String,
    /// Is this assignment currently active?
    pub enabled: bool,
}

/// Manages all control assignments in the project.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssignmentManager {
    assignments: Vec<Assignment>,
    next_id: u64,
}

impl AssignmentManager {
    /// Creates a new, empty `AssignmentManager`.
    pub fn new() -> Self {
        Self {
            assignments: Vec::new(),
            next_id: 1,
        }
    }

    /// Adds a new assignment.
    pub fn add_assignment(&mut self, assignment: Assignment) {
        self.assignments.push(assignment);
        self.next_id = self.assignments.iter().map(|a| a.id).max().unwrap_or(0) + 1;
    }

    /// Removes an assignment by its ID.
    pub fn remove_assignment(&mut self, id: u64) {
        self.assignments.retain(|a| a.id != id);
    }

    /// Returns a slice of all assignments.
    pub fn assignments(&self) -> &[Assignment] {
        &self.assignments
    }

    /// Returns a mutable slice of all assignments.
    pub fn assignments_mut(&mut self) -> &mut [Assignment] {
        &mut self.assignments
    }

    /// Generates a new unique ID for an assignment.
    pub fn generate_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}

impl Default for AssignmentManager {
    fn default() -> Self {
        Self::new()
    }
}
