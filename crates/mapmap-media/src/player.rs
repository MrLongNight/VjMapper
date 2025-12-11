//! Video playback control

use crate::{DecodedFrame, VideoDecoder};
use crossbeam_channel::{Receiver, Sender, TryRecvError};
use std::time::Duration;
use thiserror::Error;

/// Playback state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlaybackState {
    #[default]
    Idle,
    Loading,
    Playing,
    Paused,
    Stopped,
    Error(PlayerError),
}

/// Loop Mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LoopMode {
    #[default]
    Loop,
    PlayOnce,
}

/// Errors that can occur during playback
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum PlayerError {
    #[error("Failed to decode frame")]
    DecodeError,
    #[error("Failed to seek")]
    SeekError,
    #[error("Invalid state transition")]
    InvalidStateTransition,
}

/// Commands to control the video player
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlaybackCommand {
    Play,
    Pause,
    Stop,
    Seek(f64),
    SetSpeed(f32),
    SetLoopMode(LoopMode),
}

/// Video player with playback control
pub struct VideoPlayer {
    decoder: Box<dyn VideoDecoder>,
    state: PlaybackState,
    current_time: Duration,
    playback_speed: f32,
    loop_mode: LoopMode,
    last_frame: Option<DecodedFrame>,
    command_receiver: Receiver<PlaybackCommand>,
    status_sender: Sender<PlaybackState>,
}

impl VideoPlayer {
    /// Create a new video player with a decoder
    pub fn new(
        decoder: impl VideoDecoder + 'static,
        command_receiver: Receiver<PlaybackCommand>,
        status_sender: Sender<PlaybackState>,
    ) -> Self {
        Self {
            decoder: Box::new(decoder),
            state: PlaybackState::Idle,
            current_time: Duration::ZERO,
            playback_speed: 1.0,
            loop_mode: LoopMode::default(),
            last_frame: None,
            command_receiver,
            status_sender,
        }
    }

    /// Update the player (call every frame)
    pub fn update(&mut self, dt: Duration) -> Option<DecodedFrame> {
        self.handle_commands();

        if self.state == PlaybackState::Playing {
            self.current_time += dt.mul_f32(self.playback_speed);
            let duration = self.decoder.duration();

            if self.current_time >= duration {
                match self.loop_mode {
                    LoopMode::Loop => {
                        self.current_time = Duration::ZERO;
                        if self.decoder.seek(self.current_time).is_err() {
                            self.set_state(PlaybackState::Error(PlayerError::SeekError));
                        }
                    }
                    LoopMode::PlayOnce => {
                        self.set_state(PlaybackState::Stopped);
                        self.current_time = duration;
                    }
                }
            }
        }

        if self.state == PlaybackState::Playing || self.state == PlaybackState::Paused {
            match self.decoder.next_frame() {
                Ok(frame) => {
                    self.last_frame = Some(frame.clone());
                    return Some(frame);
                }
                Err(_) => {
                    self.set_state(PlaybackState::Error(PlayerError::DecodeError));
                    return self.last_frame.clone();
                }
            }
        }

        self.last_frame.clone()
    }

    fn handle_commands(&mut self) {
        match self.command_receiver.try_recv() {
            Ok(command) => self.handle_command(command),
            Err(TryRecvError::Disconnected) => {
                // Handle disconnection if necessary
            }
            Err(TryRecvError::Empty) => {}
        }
    }

    fn handle_command(&mut self, command: PlaybackCommand) {
        match command {
            PlaybackCommand::Play => self.play(),
            PlaybackCommand::Pause => self.pause(),
            PlaybackCommand::Stop => self.stop(),
            PlaybackCommand::Seek(time) => self.seek(Duration::from_secs_f64(time)),
            PlaybackCommand::SetSpeed(speed) => self.set_speed(speed),
            PlaybackCommand::SetLoopMode(mode) => self.set_loop_mode(mode),
        }
    }

    fn set_state(&mut self, new_state: PlaybackState) {
        if self.state != new_state {
            self.state = new_state;
            self.status_sender.send(self.state).ok();
        }
    }

    /// Start or resume playback
    pub fn play(&mut self) {
        match self.state {
            PlaybackState::Paused | PlaybackState::Stopped | PlaybackState::Idle => {
                self.set_state(PlaybackState::Playing);
            }
            _ => self.set_state(PlaybackState::Error(PlayerError::InvalidStateTransition)),
        }
    }

    /// Pause playback
    pub fn pause(&mut self) {
        if self.state == PlaybackState::Playing {
            self.set_state(PlaybackState::Paused);
        } else {
            self.set_state(PlaybackState::Error(PlayerError::InvalidStateTransition));
        }
    }

    /// Stop playback and reset to beginning
    pub fn stop(&mut self) {
        if self.state == PlaybackState::Playing || self.state == PlaybackState::Paused {
            self.set_state(PlaybackState::Stopped);
            self.current_time = Duration::ZERO;
            if self.decoder.seek(self.current_time).is_err() {
                self.set_state(PlaybackState::Error(PlayerError::SeekError));
            }
        } else {
            self.set_state(PlaybackState::Error(PlayerError::InvalidStateTransition));
        }
    }

    /// Seek to a specific timestamp
    pub fn seek(&mut self, timestamp: Duration) {
        if self.decoder.seek(timestamp).is_ok() {
            self.current_time = timestamp;
        } else {
            self.set_state(PlaybackState::Error(PlayerError::SeekError));
        }
    }

    /// Set playback speed
    pub fn set_speed(&mut self, speed: f32) {
        self.playback_speed = speed.clamp(0.0, 10.0);
    }

    /// Set loop mode
    pub fn set_loop_mode(&mut self, mode: LoopMode) {
        self.loop_mode = mode;
    }

    /// Get current playback state
    pub fn state(&self) -> PlaybackState {
        self.state
    }

    /// Get current playback time
    pub fn current_time(&self) -> Duration {
        self.current_time
    }

    /// Get total duration
    pub fn duration(&self) -> Duration {
        self.decoder.duration()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decoder::TestPatternDecoder;
    use crossbeam_channel::unbounded;

    fn create_test_player() -> (VideoPlayer, Sender<PlaybackCommand>, Receiver<PlaybackState>) {
        let decoder = TestPatternDecoder::new(1920, 1080, Duration::from_secs(10), 30.0);
        let (command_sender, command_receiver) = unbounded();
        let (status_sender, status_receiver) = unbounded();
        let player = VideoPlayer::new(decoder, command_receiver, status_sender);
        (player, command_sender, status_receiver)
    }

    #[test]
    fn test_initial_state() {
        let (player, _, _) = create_test_player();
        assert_eq!(player.state(), PlaybackState::Idle);
    }

    #[test]
    fn test_play_command() {
        let (mut player, command_sender, status_receiver) = create_test_player();
        command_sender.send(PlaybackCommand::Play).unwrap();
        player.handle_commands();
        assert_eq!(player.state(), PlaybackState::Playing);
        assert_eq!(status_receiver.try_recv(), Ok(PlaybackState::Playing));
    }

    #[test]
    fn test_pause_command() {
        let (mut player, command_sender, status_receiver) = create_test_player();
        player.set_state(PlaybackState::Playing);
        // Drain the receiver
        let _ = status_receiver.try_recv();
        command_sender.send(PlaybackCommand::Pause).unwrap();
        player.handle_commands();
        assert_eq!(player.state(), PlaybackState::Paused);
        assert_eq!(status_receiver.try_recv(), Ok(PlaybackState::Paused));
    }

    #[test]
    fn test_stop_command() {
        let (mut player, command_sender, status_receiver) = create_test_player();
        player.set_state(PlaybackState::Playing);
        // Drain the receiver
        let _ = status_receiver.try_recv();
        command_sender.send(PlaybackCommand::Stop).unwrap();
        player.handle_commands();
        assert_eq!(player.state(), PlaybackState::Stopped);
        assert_eq!(status_receiver.try_recv(), Ok(PlaybackState::Stopped));
        assert_eq!(player.current_time(), Duration::ZERO);
    }

    #[test]
    fn test_seek_command() {
        let (mut player, command_sender, _) = create_test_player();
        let seek_time = Duration::from_secs(5);
        command_sender
            .send(PlaybackCommand::Seek(seek_time.as_secs_f64()))
            .unwrap();
        player.handle_commands();
        assert_eq!(player.current_time(), seek_time);
    }

    #[test]
    fn test_invalid_transition_pause_from_idle() {
        let (mut player, command_sender, status_receiver) = create_test_player();
        command_sender.send(PlaybackCommand::Pause).unwrap();
        player.handle_commands();
        assert_eq!(
            player.state(),
            PlaybackState::Error(PlayerError::InvalidStateTransition)
        );
        assert_eq!(
            status_receiver.try_recv(),
            Ok(PlaybackState::Error(PlayerError::InvalidStateTransition))
        );
    }

    #[test]
    fn test_looping() {
        let (mut player, _, _) = create_test_player();
        player.set_state(PlaybackState::Playing);
        player.set_loop_mode(LoopMode::Loop);
        player.current_time = player.duration();
        player.update(Duration::from_millis(100));
        assert_eq!(player.current_time, Duration::ZERO);
    }

    #[test]
    fn test_play_once() {
        let (mut player, _, _) = create_test_player();
        player.set_state(PlaybackState::Playing);
        player.set_loop_mode(LoopMode::PlayOnce);
        player.current_time = player.duration() - Duration::from_millis(50);
        player.update(Duration::from_millis(100));
        assert_eq!(player.state(), PlaybackState::Stopped);
        assert_eq!(player.current_time, player.duration());
    }
}
