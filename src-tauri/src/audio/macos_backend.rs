/// macOS-specific audio backend using CoreAudio directly
/// This provides proper device routing that cpal doesn't support on macOS
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use coreaudio::audio_unit::{AudioUnit, IOType, Scope, Element, StreamFormat};

use super::types::{AudioError, AudioResult, PlaybackState};

const TARGET_SAMPLE_RATE: f64 = 48000.0;
const BUFFER_SIZE: usize = 512;

pub struct MacOSAudioStream {
    audio_unit: AudioUnit,
    playback_state: Arc<Mutex<PlaybackState>>,
    position: Arc<AtomicU64>,
    device_id: AudioDeviceID,
    device_name: String,
    sample_rate: f64,
}

impl MacOSAudioStream {
    /// Create a new audio stream for a specific device
    pub fn new(
        device_name: &str,
        playback_state: Arc<Mutex<PlaybackState>>,
        position: Arc<AtomicU64>,
    ) -> AudioResult<Self> {
        log::info!("Creating macOS audio stream for device: {}", device_name);

        // Find the device ID
        let device_id = Self::find_device_id(device_name)?;
        log::info!("Found device '{}' with ID: {}", device_name, device_id);

        // Create an output audio unit
        let mut audio_unit = AudioUnit::new(IOType::DefaultOutput)
            .map_err(|e| AudioError::DeviceInit(format!("Failed to create audio unit: {:?}", e)))?;

        // Set the device on the audio unit
        audio_unit.set_property(
            kAudioOutputUnitProperty_CurrentDevice,
            Scope::Global,
            Element::Output,
            Some(&device_id),
        ).map_err(|e| AudioError::DeviceInit(format!("Failed to set device: {:?}", e)))?;

        log::info!("Successfully set audio unit to use device ID: {}", device_id);

        // Initialize the audio unit without setting any format
        // Let it use the device's preferred format
        audio_unit.initialize()
            .map_err(|e| AudioError::DeviceInit(format!("Failed to initialize audio unit: {:?}", e)))?;
        log::info!("Audio unit initialized with device default format");

        // Get the actual format we ended up with
        let actual_sample_rate = if let Ok(format) = audio_unit.get_property::<StreamFormat>(
            kAudioUnitProperty_StreamFormat,
            Scope::Input,
            Element::Output,
        ) {
            log::info!("Using device format: sample_rate={}, channels={}",
                      format.sample_rate, format.channels);
            format.sample_rate
        } else {
            log::warn!("Could not get device format, assuming 48kHz");
            48000.0
        };

        Ok(Self {
            audio_unit,
            playback_state,
            position,
            device_id,
            device_name: device_name.to_string(),
            sample_rate: actual_sample_rate,
        })
    }

    /// Find a device ID by name
    fn find_device_id(device_name: &str) -> AudioResult<AudioDeviceID> {
        use coreaudio::sys::{
            kAudioHardwarePropertyDevices, kAudioObjectPropertyScopeGlobal,
            kAudioObjectSystemObject, AudioObjectGetPropertyData,
            AudioObjectGetPropertyDataSize, AudioObjectPropertyAddress,
            kAudioObjectPropertyElementMain, kAudioDevicePropertyStreams,
            kAudioDevicePropertyScopeOutput, kAudioObjectPropertyName,
        };
        use core_foundation::string::{CFString, CFStringRef};
        use core_foundation::base::TCFType;
        use std::ptr;

        unsafe {
            // Get all devices
            let property_address = AudioObjectPropertyAddress {
                mSelector: kAudioHardwarePropertyDevices,
                mScope: kAudioObjectPropertyScopeGlobal,
                mElement: kAudioObjectPropertyElementMain as u32,
            };

            let mut data_size: u32 = 0;
            let status = AudioObjectGetPropertyDataSize(
                kAudioObjectSystemObject,
                &property_address,
                0,
                ptr::null(),
                &mut data_size,
            );

            if status != 0 {
                return Err(AudioError::DeviceInit(format!("Failed to get device list size: {}", status)));
            }

            let device_count = data_size / std::mem::size_of::<AudioDeviceID>() as u32;
            let mut devices: Vec<AudioDeviceID> = vec![0; device_count as usize];

            let status = AudioObjectGetPropertyData(
                kAudioObjectSystemObject,
                &property_address,
                0,
                ptr::null(),
                &mut data_size,
                devices.as_mut_ptr() as *mut _,
            );

            if status != 0 {
                return Err(AudioError::DeviceInit(format!("Failed to get devices: {}", status)));
            }

            // Find the device by name
            for &device_id in &devices {
                // Check if it's an output device
                let output_property = AudioObjectPropertyAddress {
                    mSelector: kAudioDevicePropertyStreams,
                    mScope: kAudioDevicePropertyScopeOutput,
                    mElement: kAudioObjectPropertyElementMain as u32,
                };

                let mut stream_size: u32 = 0;
                let status = AudioObjectGetPropertyDataSize(
                    device_id,
                    &output_property,
                    0,
                    ptr::null(),
                    &mut stream_size,
                );

                if status != 0 || stream_size == 0 {
                    continue; // Not an output device
                }

                // Get device name
                let name_property = AudioObjectPropertyAddress {
                    mSelector: kAudioObjectPropertyName,
                    mScope: kAudioObjectPropertyScopeGlobal,
                    mElement: kAudioObjectPropertyElementMain as u32,
                };

                let mut cf_name: CFStringRef = ptr::null();
                let mut name_size = std::mem::size_of::<CFStringRef>() as u32;
                let status = AudioObjectGetPropertyData(
                    device_id,
                    &name_property,
                    0,
                    ptr::null(),
                    &mut name_size,
                    &mut cf_name as *mut _ as *mut _,
                );

                if status == 0 && !cf_name.is_null() {
                    let cf_string = CFString::wrap_under_get_rule(cf_name);
                    let name = cf_string.to_string();

                    if name == device_name {
                        log::info!("Found matching device: {} (ID: {})", name, device_id);
                        return Ok(device_id);
                    }
                }
            }

            Err(AudioError::DeviceInit(format!("Device '{}' not found", device_name)))
        }
    }

    /// Set the render callback
    pub fn set_render_callback<F>(&mut self, mut callback: F) -> AudioResult<()>
    where
        F: FnMut(&mut [f32]) + Send + 'static,
    {
        let playback_state = self.playback_state.clone();
        let position = self.position.clone();

        let result = self.audio_unit.set_render_callback(move |mut args: coreaudio::audio_unit::render_callback::Args<coreaudio::audio_unit::render_callback::data::NonInterleaved<f32>>| {
            // Check playback state
            let state = playback_state.lock().unwrap();
            if *state != PlaybackState::Playing {
                // Fill with silence
                for channel in args.data.channels_mut() {
                    for sample in channel {
                        *sample = 0.0;
                    }
                }
                return Ok(());
            }
            drop(state);

            // Process audio through the callback
            let num_frames = args.num_frames;

            // Create a temporary interleaved buffer
            let mut interleaved = vec![0.0f32; num_frames * 2];
            callback(&mut interleaved);

            // Copy to output buffers (non-interleaved)
            let mut channels = args.data.channels_mut();
            if let (Some(left), Some(right)) = (channels.next(), channels.next()) {
                for i in 0..num_frames {
                    left[i] = interleaved[i * 2];
                    right[i] = interleaved[i * 2 + 1];
                }
            }

            // Position is already updated by audio_callback in multi_track.rs (line 439)
            // which increments by output.len() (num_frames * 2 for stereo)
            // Do NOT increment again here or we'll skip samples causing crackling

            Ok(())
        });

        result.map_err(|e| AudioError::StreamError(format!("Failed to set render callback: {:?}", e)))
    }

    /// Initialize the audio unit (already done in new())
    pub fn initialize(&mut self) -> AudioResult<()> {
        // Already initialized in new(), just log success
        log::info!("Audio unit already initialized for device: {}", self.device_name);
        Ok(())
    }

    /// Start playback
    pub fn start(&mut self) -> AudioResult<()> {
        self.audio_unit.start()
            .map_err(|e| AudioError::StreamError(format!("Failed to start audio unit: {:?}", e)))?;
        log::info!("Audio unit started for device: {}", self.device_name);
        Ok(())
    }

    /// Stop playback
    pub fn stop(&mut self) -> AudioResult<()> {
        self.audio_unit.stop()
            .map_err(|_| AudioError::StreamError("Failed to stop audio unit".into()))?;
        self.audio_unit.uninitialize()
            .map_err(|_| AudioError::StreamError("Failed to uninitialize audio unit".into()))?;
        log::info!("Audio unit stopped");
        Ok(())
    }

    /// Get the device name
    pub fn device_name(&self) -> &str {
        &self.device_name
    }

    /// Get the device ID
    pub fn device_id(&self) -> AudioDeviceID {
        self.device_id
    }

    /// Get the sample rate
    pub fn sample_rate(&self) -> f64 {
        self.sample_rate
    }
}

impl Drop for MacOSAudioStream {
    fn drop(&mut self) {
        let _ = self.audio_unit.stop();
        let _ = self.audio_unit.uninitialize();
        log::info!("macOS audio stream dropped for device: {}", self.device_name);
    }
}

// Re-export types needed
use coreaudio::sys::{
    AudioDeviceID,
    kAudioOutputUnitProperty_CurrentDevice, kAudioUnitProperty_StreamFormat,
    kAudioFormatLinearPCM,
    kAudioFormatFlagIsFloat, kAudioFormatFlagsNativeEndian,
    kAudioFormatFlagIsPacked, kLinearPCMFormatFlagIsNonInterleaved
};