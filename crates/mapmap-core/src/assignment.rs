use serde::{Deserialize, Serialize};

/// Edge sides for edge blending
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EdgeSide {
    Left,
    Right,
    Top,
    Bottom,
}

/// A controllable parameter in the application
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ControlTarget {
    /// Layer opacity (layer_id, opacity: 0.0-1.0)
    LayerOpacity(u32),
    /// Layer position (layer_id)
    LayerPosition(u32),
    /// Layer scale (layer_id)
    LayerScale(u32),
    /// Layer rotation (layer_id, degrees)
    LayerRotation(u32),
    /// Layer visibility (layer_id)
    LayerVisibility(u32),
    /// Paint parameter (paint_id, param_name)
    PaintParameter(u32, String),
    /// Effect parameter (effect_id, param_name)
    EffectParameter(u32, String),
    /// Playback speed (global or per-layer)
    PlaybackSpeed(Option<u32>),
    /// Playback position (0.0-1.0)
    PlaybackPosition,
    /// Output brightness (output_id, brightness: 0.0-1.0)
    OutputBrightness(u32),
    /// Output edge blend (output_id, edge, width: 0.0-1.0)
    OutputEdgeBlend(u32, EdgeSide),
    /// Master opacity
    MasterOpacity,
    /// Master blackout
    MasterBlackout,
    /// Custom parameter (name)
    Custom(String),
}

/// Control value types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ControlValue {
    Float(f32),
    Int(i32),
    Bool(bool),
    String(String),
    Color(u32), // RGBA
    Vec2(f32, f32),
    Vec3(f32, f32, f32),
}

impl ControlValue {
    /// Get as float, converting if necessary
    pub fn as_float(&self) -> Option<f32> {
        match self {
            ControlValue::Float(v) => Some(*v),
            ControlValue::Int(v) => Some(*v as f32),
            ControlValue::Bool(v) => Some(if *v { 1.0 } else { 0.0 }),
            _ => None,
        }
    }

    /// Get as int, converting if necessary
    pub fn as_int(&self) -> Option<i32> {
        match self {
            ControlValue::Int(v) => Some(*v),
            ControlValue::Float(v) => Some(*v as i32),
            ControlValue::Bool(v) => Some(if *v { 1 } else { 0 }),
            _ => None,
        }
    }

    /// Get as bool, converting if necessary
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            ControlValue::Bool(v) => Some(*v),
            ControlValue::Int(v) => Some(*v != 0),
            ControlValue::Float(v) => Some(*v != 0.0),
            _ => None,
        }
    }

    /// Get as string
    pub fn as_string(&self) -> Option<&str> {
        match self {
            ControlValue::String(v) => Some(v),
            _ => None,
        }
    }
}

impl From<f32> for ControlValue {
    fn from(v: f32) -> Self {
        ControlValue::Float(v)
    }
}

impl From<i32> for ControlValue {
    fn from(v: i32) -> Self {
        ControlValue::Int(v)
    }
}

impl From<bool> for ControlValue {
    fn from(v: bool) -> Self {
        ControlValue::Bool(v)
    }
}

impl From<String> for ControlValue {
    fn from(v: String) -> Self {
        ControlValue::String(v)
    }
}

impl From<(f32, f32)> for ControlValue {
    fn from((x, y): (f32, f32)) -> Self {
        ControlValue::Vec2(x, y)
    }
}

/// A single control assignment
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Assignment {
    /// Unique ID for the control source (e.g. hash of MIDI CC channel+cc)
    pub source_id: u64,
    /// The target parameter to control
    pub target: ControlTarget,
    /// Minimum value for the target (when source is min)
    pub min: f32,
    /// Maximum value for the target (when source is max)
    pub max: f32,
    /// Whether to invert the source value
    pub invert: bool,
    /// Whether this assignment is enabled
    pub enabled: bool,
}

impl Default for Assignment {
    fn default() -> Self {
        Self {
            source_id: 0,
            target: ControlTarget::MasterOpacity,
            min: 0.0,
            max: 1.0,
            invert: false,
            enabled: true,
        }
    }
}

/// Manages all control assignments
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AssignmentManager {
    pub assignments: Vec<Assignment>,
}

impl AssignmentManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, assignment: Assignment) {
        self.assignments.push(assignment);
    }

    pub fn remove(&mut self, index: usize) {
        if index < self.assignments.len() {
            self.assignments.remove(index);
        }
    }

    pub fn find_by_source(&self, source_id: u64) -> Vec<&Assignment> {
        self.assignments.iter().filter(|a| a.source_id == source_id && a.enabled).collect()
    }
}
