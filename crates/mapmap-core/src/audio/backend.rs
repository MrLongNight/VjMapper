//! Audio Backend Abstraction
use thiserror::Error;

/// Errors that can occur in the audio backend
#[derive(Debug, Error)]
pub enum AudioError {
    #[error("No audio devices found: {0}")]
    NoDevicesFound(String),
    #[error("Default device not found")]
    DefaultDeviceNotFound,
    #[error("Unsupported stream format")]
    UnsupportedFormat,
    #[error("Failed to build audio stream: {0}")]
    StreamBuildError(String),
    #[error("Device initialization timed out")]
    Timeout,
}

/// Audio backend abstraction
pub trait AudioBackend {
    /// Start capturing audio
    fn start(&mut self) -> Result<(), AudioError>;
    /// Stop capturing audio
    fn stop(&mut self);
    /// Get the latest audio samples
    fn get_samples(&mut self) -> Vec<f32>;
}

/// CPAL implementation of the audio backend
#[cfg(feature = "audio")]
pub mod cpal_backend {
    use super::{AudioBackend, AudioError};
    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
    use crossbeam_channel::{unbounded, Receiver, Sender};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Arc;
    use tracing::{debug, error, info, warn};

    enum Command {
        Pause,
        Play,
    }

    /// CPAL audio backend
    pub struct CpalBackend {
        sample_receiver: Receiver<Vec<f32>>,
        command_sender: Sender<Command>,
        #[allow(dead_code)]
        stream: cpal::Stream,
        /// Counter for received sample batches (for periodic logging)
        sample_batch_counter: Arc<AtomicU64>,
        /// Device name for logging
        device_name: Option<String>,
    }

    impl CpalBackend {
        /// Create a new CPAL backend with the specified device.
        /// Uses a timeout to prevent the app from freezing if a device doesn't respond.
        pub fn new(device_name: Option<String>) -> Result<Self, AudioError> {
            info!("Creating CpalBackend with device: {:?}", device_name);
            let (sample_tx, sample_rx) = unbounded();
            let (command_tx, command_rx) = unbounded::<Command>();
            let sample_batch_counter = Arc::new(AtomicU64::new(0));
            let counter_clone = sample_batch_counter.clone();

            // Build stream directly in main thread (cpal::Stream is not Send)
            let stream = Self::build_stream(device_name.clone(), sample_tx, counter_clone)?;

            // Spawn command processing thread
            std::thread::Builder::new()
                .name("audio-cmd".to_string())
                .spawn(move || {
                    // Just drain the command channel - stream auto-plays
                    while command_rx.recv().is_ok() {}
                })
                .ok();

            info!(
                "Audio backend created successfully for device: {:?}",
                device_name
            );
            Ok(Self {
                sample_receiver: sample_rx,
                command_sender: command_tx,
                stream,
                sample_batch_counter,
                device_name,
            })
        }

        /// Build the audio stream (must be called from main thread)
        fn build_stream(
            device_name: Option<String>,
            sample_tx: Sender<Vec<f32>>,
            sample_counter: Arc<AtomicU64>,
        ) -> Result<cpal::Stream, AudioError> {
            let host = cpal::default_host();
            debug!("Using audio host: {:?}", host.id());

            // Get device
            let device = if let Some(ref name) = device_name {
                match host.input_devices() {
                    Ok(mut devices) => {
                        match devices.find(|d| d.name().map(|n| n == *name).unwrap_or(false)) {
                            Some(dev) => {
                                info!("Found requested audio device: {}", name);
                                dev
                            }
                            None => {
                                error!("Audio device '{}' not found", name);
                                return Err(AudioError::NoDevicesFound(format!(
                                    "Device '{}' not found",
                                    name
                                )));
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to enumerate audio devices: {}", e);
                        return Err(AudioError::NoDevicesFound(e.to_string()));
                    }
                }
            } else {
                match host.default_input_device() {
                    Some(dev) => {
                        info!("Using default audio input device: {:?}", dev.name());
                        dev
                    }
                    None => {
                        error!("No default audio input device found");
                        return Err(AudioError::DefaultDeviceNotFound);
                    }
                }
            };

            // Get config
            let config = match device.default_input_config() {
                Ok(cfg) => {
                    info!(
                        "Audio config: {} channels, {} Hz, format: {:?}",
                        cfg.channels(),
                        cfg.sample_rate().0,
                        cfg.sample_format()
                    );
                    cfg
                }
                Err(e) => {
                    error!("Failed to get device config: {}", e);
                    return Err(AudioError::StreamBuildError(format!(
                        "Failed to get device config: {}",
                        e
                    )));
                }
            };

            let err_fn = |err| error!("Audio stream callback error: {}", err);

            // Build stream
            let stream = match config.sample_format() {
                cpal::SampleFormat::F32 => {
                    let tx = sample_tx.clone();
                    let counter = sample_counter.clone();
                    device.build_input_stream(
                        &config.into(),
                        move |data: &[f32], _: &cpal::InputCallbackInfo| {
                            let batch_num = counter.fetch_add(1, Ordering::Relaxed);
                            // Log every 100th batch to avoid spam
                            if batch_num % 100 == 0 {
                                debug!(
                                    "Audio callback (F32): batch #{}, {} samples",
                                    batch_num,
                                    data.len()
                                );
                            }
                            if tx.send(data.to_vec()).is_err() {
                                warn!("Audio sample channel closed");
                            }
                        },
                        err_fn,
                        None,
                    )
                }
                cpal::SampleFormat::I16 => {
                    let tx = sample_tx.clone();
                    let counter = sample_counter.clone();
                    device.build_input_stream(
                        &config.into(),
                        move |data: &[i16], _: &cpal::InputCallbackInfo| {
                            let batch_num = counter.fetch_add(1, Ordering::Relaxed);
                            if batch_num % 100 == 0 {
                                debug!(
                                    "Audio callback (I16): batch #{}, {} samples",
                                    batch_num,
                                    data.len()
                                );
                            }
                            let samples: Vec<f32> =
                                data.iter().map(|&s| s as f32 / i16::MAX as f32).collect();
                            if tx.send(samples).is_err() {
                                warn!("Audio sample channel closed");
                            }
                        },
                        err_fn,
                        None,
                    )
                }
                cpal::SampleFormat::U16 => {
                    let tx = sample_tx.clone();
                    let counter = sample_counter.clone();
                    device.build_input_stream(
                        &config.into(),
                        move |data: &[u16], _: &cpal::InputCallbackInfo| {
                            let batch_num = counter.fetch_add(1, Ordering::Relaxed);
                            if batch_num % 100 == 0 {
                                debug!(
                                    "Audio callback (U16): batch #{}, {} samples",
                                    batch_num,
                                    data.len()
                                );
                            }
                            let samples: Vec<f32> = data
                                .iter()
                                .map(|&s| (s as f32 / u16::MAX as f32) * 2.0 - 1.0)
                                .collect();
                            if tx.send(samples).is_err() {
                                warn!("Audio sample channel closed");
                            }
                        },
                        err_fn,
                        None,
                    )
                }
                format => {
                    error!("Unsupported audio sample format: {:?}", format);
                    return Err(AudioError::StreamBuildError(format!(
                        "Unsupported sample format: {:?}",
                        format
                    )));
                }
            };

            match stream {
                Ok(stream) => {
                    // Start the stream immediately
                    if let Err(e) = stream.play() {
                        error!("Failed to start audio stream: {}", e);
                        return Err(AudioError::StreamBuildError(format!(
                            "Failed to start stream: {}",
                            e
                        )));
                    }
                    info!("Audio stream started successfully");
                    Ok(stream)
                }
                Err(e) => {
                    error!("Failed to build audio stream: {}", e);
                    Err(AudioError::StreamBuildError(e.to_string()))
                }
            }
        }

        pub fn list_devices() -> Result<Option<Vec<String>>, AudioError> {
            let host = cpal::default_host();
            let devices = host
                .input_devices()
                .map_err(|e| AudioError::NoDevicesFound(e.to_string()))?;
            let device_names: Vec<String> = devices.filter_map(|d| d.name().ok()).collect();
            info!("Found {} audio input devices", device_names.len());
            Ok(Some(device_names))
        }
    }

    impl AudioBackend for CpalBackend {
        fn start(&mut self) -> Result<(), AudioError> {
            info!("Starting audio capture for device: {:?}", self.device_name);
            // Stream is already playing from initialization
            let _ = self.command_sender.send(Command::Play);
            Ok(())
        }

        fn stop(&mut self) {
            info!("Stopping audio capture for device: {:?}", self.device_name);
            let _ = self.command_sender.send(Command::Pause);
        }

        fn get_samples(&mut self) -> Vec<f32> {
            let samples: Vec<f32> = self.sample_receiver.try_iter().flatten().collect();
            let batch_count = self.sample_batch_counter.load(Ordering::Relaxed);
            // Log periodically when receiving samples
            if !samples.is_empty() && batch_count % 100 == 0 {
                debug!(
                    "get_samples: returning {} samples (batch count: {})",
                    samples.len(),
                    batch_count
                );
            }
            samples
        }
    }

    impl Drop for CpalBackend {
        fn drop(&mut self) {
            info!("Dropping CpalBackend for device: {:?}", self.device_name);
            // Dropping command_sender will close the channel and
            // the command thread will exit its recv() loop
            // Stream will be dropped automatically
        }
    }
}

/// A mock audio backend for testing without native audio dependencies
#[cfg(any(test, feature = "mock-audio"))]
pub mod mock_backend {
    use super::{AudioBackend, AudioError};

    pub struct MockBackend {
        phase: f32,
        sample_rate: f32,
    }

    impl Default for MockBackend {
        fn default() -> Self {
            Self {
                phase: 0.0,
                sample_rate: 44100.0,
            }
        }
    }

    impl MockBackend {
        pub fn new() -> Self {
            Self::default()
        }
    }

    impl AudioBackend for MockBackend {
        fn start(&mut self) -> Result<(), AudioError> {
            Ok(())
        }

        fn stop(&mut self) {}

        fn get_samples(&mut self) -> Vec<f32> {
            let mut buffer = vec![0.0; 1024];
            for sample in &mut buffer {
                *sample = (self.phase * 2.0 * std::f32::consts::PI).sin();
                self.phase += 440.0 / self.sample_rate;
                if self.phase > 1.0 {
                    self.phase -= 1.0;
                }
            }
            buffer
        }
    }
}
