//! Audio-Media Pipeline Integration
//!
//! This module connects the audio backend to the media pipeline,
//! allowing audio analysis to drive visual effects in real-time.

use crate::audio::{AudioAnalyzer, AudioConfig};
use crate::audio_reactive::AudioReactiveController;
use crossbeam_channel::{Receiver, Sender};
use parking_lot::RwLock;
use std::sync::Arc;

/// Audio pipeline that integrates with media processing
pub struct AudioMediaPipeline {
    /// Audio analyzer for FFT and beat detection
    analyzer: Arc<RwLock<AudioAnalyzer>>,

    /// Audio-reactive controller for parameter mapping
    reactive_controller: Arc<RwLock<AudioReactiveController>>,

    /// Channel to send audio samples to analyzer
    sample_sender: Sender<Vec<f32>>,

    /// Channel to receive analyzed data
    analysis_receiver: Receiver<crate::audio::AudioAnalysis>,

    /// Latency compensation in milliseconds
    latency_ms: f32,
}

impl AudioMediaPipeline {
    /// Create a new audio-media pipeline
    pub fn new(config: AudioConfig) -> Self {
        let analyzer = Arc::new(RwLock::new(AudioAnalyzer::new(config)));
        let reactive_controller = Arc::new(RwLock::new(AudioReactiveController::new()));

        let (sample_tx, sample_rx) = crossbeam_channel::unbounded();
        let analysis_rx = analyzer.read().analysis_receiver();

        // Spawn audio processing thread
        let analyzer_clone = analyzer.clone();
        std::thread::Builder::new()
            .name("audio-processor".to_string())
            .spawn(move || {
                let mut timestamp = 0.0;
                let dt = 1.0 / 44100.0; // Assuming 44.1kHz sample rate

                while let Ok(samples) = sample_rx.recv() {
                    let mut analyzer = analyzer_clone.write();
                    analyzer.process_samples(&samples, timestamp);
                    timestamp += samples.len() as f64 * dt;
                }
            })
            .expect("Failed to spawn audio processor thread");

        Self {
            analyzer,
            reactive_controller,
            sample_sender: sample_tx,
            analysis_receiver: analysis_rx,
            latency_ms: 0.0,
        }
    }

    /// Send audio samples to the pipeline
    pub fn process_samples(&self, samples: &[f32]) {
        let _ = self.sample_sender.send(samples.to_vec());
    }

    /// Get the latest audio analysis
    pub fn get_analysis(&self) -> Option<crate::audio::AudioAnalysis> {
        self.analysis_receiver.try_recv().ok()
    }

    /// Set latency compensation
    pub fn set_latency_compensation(&mut self, latency_ms: f32) {
        self.latency_ms = latency_ms;
    }

    /// Get latency compensation
    pub fn latency_compensation(&self) -> f32 {
        self.latency_ms
    }

    /// Get audio-reactive controller
    pub fn reactive_controller(&self) -> Arc<RwLock<AudioReactiveController>> {
        self.reactive_controller.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_media_pipeline_creation() {
        let config = AudioConfig::default();
        let pipeline = AudioMediaPipeline::new(config);
        assert_eq!(pipeline.latency_compensation(), 0.0);
    }

    #[test]
    fn test_latency_compensation() {
        let config = AudioConfig::default();
        let mut pipeline = AudioMediaPipeline::new(config);

        pipeline.set_latency_compensation(50.0);
        assert_eq!(pipeline.latency_compensation(), 50.0);
    }
}
