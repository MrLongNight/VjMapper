use serde::{Deserialize, Serialize};

/// Identifies a control source
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ControlSource {
    /// MIDI Control Change (channel, cc)
    MidiCC(u8, u8),
    /// MIDI Note (channel, note)
    MidiNote(u8, u8),
    /// OSC Address (address)
    OscAddress(String),
}

impl ControlSource {
    /// Generate a stable ID for this source
    pub fn id(&self) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }

    /// Create a human-readable label
    pub fn label(&self) -> String {
        match self {
            ControlSource::MidiCC(ch, cc) => format!("MIDI Ch {} CC {}", ch + 1, cc),
            ControlSource::MidiNote(ch, note) => format!("MIDI Ch {} Note {}", ch + 1, note),
            ControlSource::OscAddress(addr) => format!("OSC {}", addr),
        }
    }
}
