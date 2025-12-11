//! Phase 6: Dashboard Controls
//!
//! Quick-access parameter controls for media playback.

use crossbeam_channel::Sender;
use imgui::{im_str, Condition, Ui};
use mapmap_media::{LoopMode, PlaybackCommand, PlaybackState};
use std::time::Duration;

/// Dashboard control panel for media playback
pub struct Dashboard {
    playback_state: PlaybackState,
    current_time: Duration,
    duration: Duration,
    speed: f32,
    loop_mode: LoopMode,
    command_sender: Sender<PlaybackCommand>,
}

impl Dashboard {
    pub fn new(command_sender: Sender<PlaybackCommand>) -> Self {
        Self {
            playback_state: PlaybackState::Idle,
            current_time: Duration::ZERO,
            duration: Duration::ZERO,
            speed: 1.0,
            loop_mode: LoopMode::default(),
            command_sender,
        }
    }

    /// Update the dashboard's state from the media player
    pub fn update_state(
        &mut self,
        state: PlaybackState,
        current_time: Duration,
        duration: Duration,
    ) {
        self.playback_state = state;
        self.current_time = current_time;
        self.duration = duration;
    }

    /// Render the dashboard UI
    pub fn ui(&mut self, ui: &Ui) {
        ui.window(im_str!("Dashboard"))
            .size([400.0, 200.0], Condition::FirstUseEver)
            .build(|| {
                // Playback controls
                if ui.button(im_str!("Play"), [50.0, 20.0]) {
                    self.command_sender.send(PlaybackCommand::Play).ok();
                }
                ui.same_line();
                if ui.button(im_str!("Pause"), [50.0, 20.0]) {
                    self.command_sender.send(PlaybackCommand::Pause).ok();
                }
                ui.same_line();
                if ui.button(im_str!("Stop"), [50.0, 20.0]) {
                    self.command_sender.send(PlaybackCommand::Stop).ok();
                }

                // Timeline scrubber
                let total_secs = self.duration.as_secs_f32();
                let mut current_pos = self.current_time.as_secs_f32();
                if ui.slider(
                    im_str!("Timeline"),
                    0.0,
                    total_secs,
                    &mut current_pos,
                ) {
                    self.command_sender
                        .send(PlaybackCommand::Seek(current_pos as f64))
                        .ok();
                }

                // Speed control
                if ui.slider(im_str!("Speed"), 0.1, 4.0, &mut self.speed) {
                    self.command_sender
                        .send(PlaybackCommand::SetSpeed(self.speed))
                        .ok();
                }

                // Loop mode
                let mut loop_mode_changed = false;
                if ui.radio_button(
                    im_str!("Loop"),
                    &mut self.loop_mode,
                    LoopMode::Loop,
                ) {
                    loop_mode_changed = true;
                }
                ui.same_line();
                if ui.radio_button(
                    im_str!("Play Once"),
                    &mut self.loop_mode,
                    LoopMode::PlayOnce,
                ) {
                    loop_mode_changed = true;
                }

                if loop_mode_changed {
                    self.command_sender
                        .send(PlaybackCommand::SetLoopMode(self.loop_mode))
                        .ok();
                }

                // Status display
                ui.separator();
                ui.text(im_str!("State: {:?}", self.playback_state));
                ui.text(im_str!(
                    "Time: {:.2}/{:.2}",
                    self.current_time.as_secs_f32(),
                    total_secs
                ));
            });
    }
}
