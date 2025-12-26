//! MIDI Function Assignment Table
//!
//! Allows users to manually assign MapFlow, Mixxx, and Streamer.bot
//! functions to MIDI controls.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Target application for function assignment
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssignmentTarget {
    /// MapFlow internal parameter
    MapFlow,
    /// Mixxx DJ software
    Mixxx,
    /// Streamer.bot
    StreamerBot,
    /// No assignment
    None,
}

impl Default for AssignmentTarget {
    fn default() -> Self {
        Self::None
    }
}

/// A single function assignment for a MIDI control
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionAssignment {
    /// MIDI element ID (e.g., "encoder_1", "ch2_gain")
    pub element_id: String,

    /// Target application
    pub target: AssignmentTarget,

    /// Function name/identifier in target app
    pub function: String,

    /// Human-readable description
    pub description: String,

    /// Additional parameters (JSON object)
    #[serde(default)]
    pub params: HashMap<String, String>,

    /// Is this assignment enabled?
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool {
    true
}

impl FunctionAssignment {
    pub fn new(element_id: &str) -> Self {
        Self {
            element_id: element_id.to_string(),
            target: AssignmentTarget::None,
            function: String::new(),
            description: String::new(),
            params: HashMap::new(),
            enabled: true,
        }
    }

    pub fn with_mapflow(mut self, function: &str, description: &str) -> Self {
        self.target = AssignmentTarget::MapFlow;
        self.function = function.to_string();
        self.description = description.to_string();
        self
    }

    pub fn with_mixxx(mut self, function: &str, description: &str) -> Self {
        self.target = AssignmentTarget::Mixxx;
        self.function = function.to_string();
        self.description = description.to_string();
        self
    }

    pub fn with_streamerbot(mut self, action: &str, description: &str) -> Self {
        self.target = AssignmentTarget::StreamerBot;
        self.function = action.to_string();
        self.description = description.to_string();
        self
    }
}

/// Complete assignment table for a controller
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AssignmentTable {
    /// Controller name
    pub controller: String,

    /// All assignments
    pub assignments: Vec<FunctionAssignment>,

    /// Last modified timestamp
    #[serde(default)]
    pub last_modified: String,
}

impl AssignmentTable {
    pub fn new(controller: &str) -> Self {
        Self {
            controller: controller.to_string(),
            assignments: Vec::new(),
            last_modified: chrono_now(),
        }
    }

    /// Add or update an assignment
    pub fn set(&mut self, assignment: FunctionAssignment) {
        if let Some(existing) = self
            .assignments
            .iter_mut()
            .find(|a| a.element_id == assignment.element_id)
        {
            *existing = assignment;
        } else {
            self.assignments.push(assignment);
        }
        self.last_modified = chrono_now();
    }

    /// Get assignment for element
    pub fn get(&self, element_id: &str) -> Option<&FunctionAssignment> {
        self.assignments.iter().find(|a| a.element_id == element_id)
    }

    /// Get mutable assignment for element
    pub fn get_mut(&mut self, element_id: &str) -> Option<&mut FunctionAssignment> {
        self.assignments
            .iter_mut()
            .find(|a| a.element_id == element_id)
    }

    /// Remove assignment
    pub fn remove(&mut self, element_id: &str) -> bool {
        let len = self.assignments.len();
        self.assignments.retain(|a| a.element_id != element_id);
        self.last_modified = chrono_now();
        self.assignments.len() < len
    }

    /// Get all assignments for a target
    pub fn by_target(&self, target: &AssignmentTarget) -> Vec<&FunctionAssignment> {
        self.assignments
            .iter()
            .filter(|a| &a.target == target)
            .collect()
    }

    /// Load from JSON file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, std::io::Error> {
        let content = fs::read_to_string(path)?;
        serde_json::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    /// Save to JSON file
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), std::io::Error> {
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        fs::write(path, content)
    }

    /// Create default Ecler NUO 4 assignments
    pub fn ecler_nuo4_defaults() -> Self {
        let mut table = Self::new("Ecler NUO 4");

        // Channel 2 defaults
        table.set(
            FunctionAssignment::new("ch2_gain").with_mapflow("layer_opacity_0", "Layer 1 Opacity"),
        );
        table.set(
            FunctionAssignment::new("ch2_treble")
                .with_mapflow("effect_param_0_treble", "Effect 1 Treble"),
        );
        table.set(
            FunctionAssignment::new("ch2_mid").with_mapflow("effect_param_0_mid", "Effect 1 Mid"),
        );
        table.set(
            FunctionAssignment::new("ch2_bass")
                .with_mapflow("effect_param_0_bass", "Effect 1 Bass"),
        );
        table.set(
            FunctionAssignment::new("ch2_fader").with_mixxx("[Channel1],volume", "Deck 1 Volume"),
        );

        // Channel 3 defaults
        table.set(
            FunctionAssignment::new("ch3_gain").with_mapflow("layer_opacity_1", "Layer 2 Opacity"),
        );
        table.set(
            FunctionAssignment::new("ch3_fader").with_mixxx("[Channel2],volume", "Deck 2 Volume"),
        );

        // Crossfader
        table.set(
            FunctionAssignment::new("crossfader").with_mixxx("[Master],crossfader", "Crossfader"),
        );

        // MIDI Control section - Encoder 1-4 (Layout 1, Bank A)
        table.set(
            FunctionAssignment::new("encoder_1").with_mapflow("master_opacity", "Master Opacity"),
        );
        table
            .set(FunctionAssignment::new("encoder_2").with_mapflow("master_speed", "Master Speed"));
        table
            .set(FunctionAssignment::new("encoder_3").with_mapflow("effect_mix_0", "Effect 1 Mix"));
        table
            .set(FunctionAssignment::new("encoder_4").with_mapflow("effect_mix_1", "Effect 2 Mix"));

        // Switches - Streamer.bot examples
        table.set(
            FunctionAssignment::new("switch_1")
                .with_streamerbot("Scene_Change_Main", "Switch to Main Scene"),
        );
        table.set(
            FunctionAssignment::new("switch_2")
                .with_streamerbot("Scene_Change_BRB", "Switch to BRB Scene"),
        );
        table.set(
            FunctionAssignment::new("switch_3").with_streamerbot("Toggle_Mute", "Toggle Mic Mute"),
        );
        table.set(
            FunctionAssignment::new("switch_4")
                .with_streamerbot("Trigger_Alert", "Trigger Custom Alert"),
        );

        table
    }
}

/// Get current timestamp as string
fn chrono_now() -> String {
    // Timestamp feature disabled - return empty string
    String::new()
}

/// Predefined MapFlow functions
pub const MAPFLOW_FUNCTIONS: &[(&str, &str)] = &[
    ("master_opacity", "Master Opacity"),
    ("master_speed", "Master Speed"),
    ("layer_opacity_0", "Layer 1 Opacity"),
    ("layer_opacity_1", "Layer 2 Opacity"),
    ("layer_opacity_2", "Layer 3 Opacity"),
    ("layer_opacity_3", "Layer 4 Opacity"),
    ("layer_speed_0", "Layer 1 Speed"),
    ("layer_speed_1", "Layer 2 Speed"),
    ("effect_mix_0", "Effect 1 Mix"),
    ("effect_mix_1", "Effect 2 Mix"),
    ("effect_param_0_intensity", "Effect 1 Intensity"),
    ("effect_param_1_intensity", "Effect 2 Intensity"),
    ("bpm_tap", "BPM Tap"),
    ("bpm_sync", "BPM Sync"),
    ("blackout", "Blackout"),
    ("flash", "Flash"),
    ("next_cue", "Next Cue"),
    ("prev_cue", "Previous Cue"),
    ("trigger_cue_0", "Trigger Cue 1"),
    ("trigger_cue_1", "Trigger Cue 2"),
];

/// Predefined Mixxx functions (common ones)
pub const MIXXX_FUNCTIONS: &[(&str, &str)] = &[
    ("[Channel1],volume", "Deck 1 Volume"),
    ("[Channel2],volume", "Deck 2 Volume"),
    ("[Channel1],play", "Deck 1 Play"),
    ("[Channel2],play", "Deck 2 Play"),
    ("[Channel1],cue_default", "Deck 1 Cue"),
    ("[Channel2],cue_default", "Deck 2 Cue"),
    ("[Channel1],sync_enabled", "Deck 1 Sync"),
    ("[Channel2],sync_enabled", "Deck 2 Sync"),
    ("[Channel1],rate", "Deck 1 Pitch"),
    ("[Channel2],rate", "Deck 2 Pitch"),
    ("[Channel1],filterHigh", "Deck 1 Treble EQ"),
    ("[Channel1],filterMid", "Deck 1 Mid EQ"),
    ("[Channel1],filterLow", "Deck 1 Bass EQ"),
    ("[Channel2],filterHigh", "Deck 2 Treble EQ"),
    ("[Channel2],filterMid", "Deck 2 Mid EQ"),
    ("[Channel2],filterLow", "Deck 2 Bass EQ"),
    ("[Master],crossfader", "Crossfader"),
    ("[Master],headMix", "Headphone Mix"),
    ("[EffectRack1_EffectUnit1],mix", "FX Unit 1 Mix"),
    ("[EffectRack1_EffectUnit2],mix", "FX Unit 2 Mix"),
];

/// Predefined Streamer.bot actions (examples)
pub const STREAMERBOT_ACTIONS: &[(&str, &str)] = &[
    ("Scene_Change_Main", "Switch to Main Scene"),
    ("Scene_Change_BRB", "Switch to BRB Scene"),
    ("Scene_Change_Starting", "Switch to Starting Scene"),
    ("Scene_Change_Ending", "Switch to Ending Scene"),
    ("Toggle_Mute", "Toggle Mic Mute"),
    ("Toggle_Deafen", "Toggle Deafen"),
    ("Trigger_Alert", "Trigger Custom Alert"),
    ("Play_Sound_Effect", "Play Sound Effect"),
    ("Start_Timer", "Start Timer"),
    ("Stop_Timer", "Stop Timer"),
    ("Send_Chat_Message", "Send Chat Message"),
    ("Change_Title", "Change Stream Title"),
    ("Run_Ad", "Run Ad"),
    ("Create_Marker", "Create Stream Marker"),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assignment_table_crud() {
        let mut table = AssignmentTable::new("Test");

        // Create
        table.set(FunctionAssignment::new("test_knob").with_mapflow("master_opacity", "Test"));

        // Read
        assert!(table.get("test_knob").is_some());
        assert_eq!(table.get("test_knob").unwrap().function, "master_opacity");

        // Update
        table.set(FunctionAssignment::new("test_knob").with_mixxx("[Channel1],volume", "Updated"));
        assert_eq!(
            table.get("test_knob").unwrap().target,
            AssignmentTarget::Mixxx
        );

        // Delete
        assert!(table.remove("test_knob"));
        assert!(table.get("test_knob").is_none());
    }

    #[test]
    fn test_ecler_nuo4_defaults() {
        let table = AssignmentTable::ecler_nuo4_defaults();

        assert_eq!(table.controller, "Ecler NUO 4");
        assert!(!table.assignments.is_empty());

        // Check some defaults
        let crossfader = table.get("crossfader");
        assert!(crossfader.is_some());
        assert_eq!(crossfader.unwrap().target, AssignmentTarget::Mixxx);
    }

    #[test]
    fn test_by_target() {
        let table = AssignmentTable::ecler_nuo4_defaults();

        let mapflow = table.by_target(&AssignmentTarget::MapFlow);
        let mixxx = table.by_target(&AssignmentTarget::Mixxx);
        let streamerbot = table.by_target(&AssignmentTarget::StreamerBot);

        assert!(!mapflow.is_empty());
        assert!(!mixxx.is_empty());
        assert!(!streamerbot.is_empty());
    }

    #[test]
    fn test_serialization() {
        let table = AssignmentTable::ecler_nuo4_defaults();

        let json = serde_json::to_string_pretty(&table).unwrap();
        let loaded: AssignmentTable = serde_json::from_str(&json).unwrap();

        assert_eq!(table.controller, loaded.controller);
        assert_eq!(table.assignments.len(), loaded.assignments.len());
    }
}
