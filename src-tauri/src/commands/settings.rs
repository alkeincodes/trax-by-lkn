use tauri::State;
use serde::{Serialize, Deserialize};

#[cfg(not(target_os = "macos"))]
use cpal::traits::{HostTrait, DeviceTrait};

use super::AppState;
use crate::database::AppSettings;

#[derive(Serialize, Deserialize)]
pub struct AudioDevice {
  pub name: String,
  pub is_default: bool,
}

#[cfg(target_os = "macos")]
#[tauri::command]
pub fn get_audio_devices() -> Result<Vec<AudioDevice>, String> {
  use coreaudio::sys::{
    kAudioHardwarePropertyDevices, kAudioObjectPropertyScopeGlobal,
    kAudioObjectSystemObject, AudioObjectGetPropertyData,
    AudioObjectGetPropertyDataSize, AudioObjectPropertyAddress,
    kAudioObjectPropertyElementMain, kAudioDevicePropertyStreams,
    kAudioDevicePropertyScopeOutput, kAudioObjectPropertyName,
    kAudioHardwarePropertyDefaultOutputDevice, AudioDeviceID
  };
  use core_foundation::string::{CFString, CFStringRef};
  use core_foundation::base::TCFType;
  use std::ptr;

  log::info!("Enumerating audio output devices (macOS)...");

  let mut audio_devices = Vec::new();

  unsafe {
    // Get default device ID first
    let mut default_device_id: AudioDeviceID = 0;
    let default_property = AudioObjectPropertyAddress {
      mSelector: kAudioHardwarePropertyDefaultOutputDevice,
      mScope: kAudioObjectPropertyScopeGlobal,
      mElement: kAudioObjectPropertyElementMain as u32,
    };

    let mut size = std::mem::size_of::<AudioDeviceID>() as u32;
    let _ = AudioObjectGetPropertyData(
      kAudioObjectSystemObject,
      &default_property,
      0,
      ptr::null(),
      &mut size,
      &mut default_device_id as *mut _ as *mut _,
    );

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
      return Err(format!("Failed to get device list size: {}", status));
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
      return Err(format!("Failed to get devices: {}", status));
    }

    // Iterate through all devices
    for &device_id in &devices {
      // Check if this is an output device
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

      // Skip if not an output device
      if status != 0 || stream_size == 0 {
        continue;
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

        log::info!("Found audio device: {} (ID: {}, Default: {})",
                   name, device_id, device_id == default_device_id);

        audio_devices.push(AudioDevice {
          name,
          is_default: device_id == default_device_id,
        });
      }
    }
  }

  // Sort so default is first
  audio_devices.sort_by(|a, b| b.is_default.cmp(&a.is_default));

  log::info!("Total devices found: {}", audio_devices.len());
  Ok(audio_devices)
}

#[cfg(not(target_os = "macos"))]
#[tauri::command]
pub fn get_audio_devices() -> Result<Vec<AudioDevice>, String> {
  let host = cpal::default_host();

  log::info!("Enumerating audio output devices...");

  // Get the default device first
  let default_device = host.default_output_device();
  let default_device_name = default_device.as_ref().and_then(|d| d.name().ok());

  log::info!("Default device: {:?}", default_device_name);

  let mut audio_devices = Vec::new();
  let mut device_names = std::collections::HashSet::new();

  // Always add the default device first if it exists
  if let Some(ref name) = default_device_name {
    log::info!("Adding default device: {}", name);
    audio_devices.push(AudioDevice {
      name: name.clone(),
      is_default: true,
    });
    device_names.insert(name.clone());
  }

  // Enumerate all other output devices
  if let Ok(devices) = host.output_devices() {
    for device in devices {
      if let Ok(name) = device.name() {
        // Only add if we haven't already added this device
        if device_names.insert(name.clone()) {
          log::info!("Found additional audio device: {}", name);
          audio_devices.push(AudioDevice {
            name,
            is_default: false,
          });
        }
      } else {
        log::warn!("Failed to get name for audio device");
      }
    }
  }

  log::info!("Total devices found: {}", audio_devices.len());

  Ok(audio_devices)
}

#[tauri::command]
pub fn get_audio_settings(state: State<'_, AppState>) -> Result<AppSettings, String> {
  state.database
    .get_settings()
    .map_err(|e| format!("Failed to get audio settings: {}", e))
}

#[tauri::command]
pub fn set_audio_device(
  state: State<'_, AppState>,
  device_name: String,
) -> Result<(), String> {
  let mut settings = state.database
    .get_settings()
    .map_err(|e| format!("Failed to get settings: {}", e))?;

  settings.audio_output_device = Some(device_name.clone());

  state.database
    .update_settings(&settings)
    .map_err(|e| format!("Failed to update audio device: {}", e))?;

  log::info!("Audio output device set to: {}", device_name);
  Ok(())
}

#[tauri::command]
pub fn set_buffer_size(
  state: State<'_, AppState>,
  buffer_size: i32,
) -> Result<(), String> {
  let mut settings = state.database
    .get_settings()
    .map_err(|e| format!("Failed to get settings: {}", e))?;

  settings.audio_buffer_size = buffer_size;

  state.database
    .update_settings(&settings)
    .map_err(|e| format!("Failed to update buffer size: {}", e))?;

  log::info!("Audio buffer size set to: {}", buffer_size);
  Ok(())
}

#[tauri::command]
pub fn set_sample_rate(
  state: State<'_, AppState>,
  sample_rate: i32,
) -> Result<(), String> {
  let mut settings = state.database
    .get_settings()
    .map_err(|e| format!("Failed to get settings: {}", e))?;

  settings.sample_rate = sample_rate;

  state.database
    .update_settings(&settings)
    .map_err(|e| format!("Failed to update sample rate: {}", e))?;

  log::info!("Sample rate set to: {}", sample_rate);
  Ok(())
}

#[tauri::command]
pub fn switch_audio_device(
  state: State<'_, AppState>,
  device_name: String,
) -> Result<(), String> {
  // First save the setting to database
  let mut settings = state.database
    .get_settings()
    .map_err(|e| format!("Failed to get settings: {}", e))?;

  settings.audio_output_device = Some(device_name.clone());

  state.database
    .update_settings(&settings)
    .map_err(|e| format!("Failed to update audio device: {}", e))?;

  // Then switch the audio engine to the new device
  let mut engine = state.audio_engine.lock().unwrap();
  engine.switch_audio_device(&device_name)
    .map_err(|e| format!("Failed to switch audio device: {}", e))?;

  log::info!("Audio output device switched to: {}", device_name);
  Ok(())
}

/// Get the current audio output device name
#[tauri::command]
pub fn get_current_audio_device(state: State<'_, AppState>) -> Result<Option<String>, String> {
  let engine = state.audio_engine.lock()
    .map_err(|_| "Failed to lock audio engine".to_string())?;

  Ok(engine.current_device_name())
}
