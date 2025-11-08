use std::path::{Path, PathBuf};
use std::fs;
use symphonia::core::audio::{AudioBufferRef, Signal};
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use hound::{WavWriter, WavSpec};

use super::ImportError;

/// Get the app data directory for storing mixdowns
/// Works on both Windows and macOS
pub fn get_mixdowns_directory() -> Result<PathBuf, ImportError> {
  // Get the app data directory based on platform
  let app_data = if cfg!(target_os = "windows") {
    // Windows: C:\Users\<user>\AppData\Local\TraX\mixdowns
    std::env::var("LOCALAPPDATA")
      .map(PathBuf::from)
      .map_err(|_| ImportError::Io(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "Could not find LOCALAPPDATA directory"
      )))?
      .join("TraX")
  } else if cfg!(target_os = "macos") {
    // macOS: ~/Library/Application Support/TraX/mixdowns
    dirs::data_local_dir()
      .ok_or_else(|| ImportError::Io(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "Could not find Application Support directory"
      )))?
      .join("TraX")
  } else {
    // Linux fallback: ~/.local/share/TraX/mixdowns
    dirs::data_local_dir()
      .ok_or_else(|| ImportError::Io(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "Could not find data directory"
      )))?
      .join("TraX")
  };

  let mixdowns_dir = app_data.join("mixdowns");

  // Create directory if it doesn't exist
  if !mixdowns_dir.exists() {
    fs::create_dir_all(&mixdowns_dir)?;
  }

  Ok(mixdowns_dir)
}

/// Generate a mixdown filename based on song ID
pub fn get_mixdown_filename(song_id: &str) -> String {
  format!("{}.wav", song_id)
}

/// Decode an audio file and return its samples as f32 vectors
fn decode_audio_file(file_path: &Path) -> Result<(Vec<f32>, Vec<f32>, u32), ImportError> {
  let file = std::fs::File::open(file_path)
    .map_err(|e| ImportError::Io(e))?;

  let mss = MediaSourceStream::new(Box::new(file), Default::default());

  let mut hint = Hint::new();
  if let Some(ext) = file_path.extension().and_then(|e| e.to_str()) {
    hint.with_extension(ext);
  }

  let format_opts = FormatOptions::default();
  let metadata_opts = MetadataOptions::default();

  let probed = symphonia::default::get_probe()
    .format(&hint, mss, &format_opts, &metadata_opts)
    .map_err(|e| ImportError::MetadataExtraction(format!("Failed to probe file: {}", e)))?;

  let mut format = probed.format;
  let track = format.default_track()
    .ok_or_else(|| ImportError::InvalidFormat("No audio track found".to_string()))?;

  let track_id = track.id;
  let sample_rate = track.codec_params.sample_rate
    .ok_or_else(|| ImportError::InvalidFormat("No sample rate found".to_string()))?;

  let mut decoder = symphonia::default::get_codecs()
    .make(&track.codec_params, &DecoderOptions::default())
    .map_err(|e| ImportError::InvalidFormat(format!("Failed to create decoder: {}", e)))?;

  let mut left_channel = Vec::new();
  let mut right_channel = Vec::new();

  // Decode all packets
  loop {
    let packet = match format.next_packet() {
      Ok(packet) => packet,
      Err(_) => break,
    };

    if packet.track_id() != track_id {
      continue;
    }

    match decoder.decode(&packet) {
      Ok(decoded) => {
        // Convert samples to f32
        match decoded {
          AudioBufferRef::F32(buf) => {
            let channels = buf.spec().channels.count();
            if channels == 1 {
              // Mono: duplicate to both channels
              let samples = buf.chan(0);
              left_channel.extend_from_slice(samples);
              right_channel.extend_from_slice(samples);
            } else if channels >= 2 {
              // Stereo or more: take first two channels
              let left = buf.chan(0);
              let right = buf.chan(1);
              left_channel.extend_from_slice(left);
              right_channel.extend_from_slice(right);
            }
          }
          AudioBufferRef::S16(buf) => {
            let channels = buf.spec().channels.count();
            if channels == 1 {
              let samples: Vec<f32> = buf.chan(0).iter().map(|&s| s as f32 / 32768.0).collect();
              left_channel.extend_from_slice(&samples);
              right_channel.extend_from_slice(&samples);
            } else if channels >= 2 {
              let left: Vec<f32> = buf.chan(0).iter().map(|&s| s as f32 / 32768.0).collect();
              let right: Vec<f32> = buf.chan(1).iter().map(|&s| s as f32 / 32768.0).collect();
              left_channel.extend_from_slice(&left);
              right_channel.extend_from_slice(&right);
            }
          }
          _ => {
            return Err(ImportError::InvalidFormat("Unsupported audio format".to_string()));
          }
        }
      }
      Err(e) => {
        log::warn!("Decode error: {}", e);
        break;
      }
    }
  }

  Ok((left_channel, right_channel, sample_rate))
}

/// Generate a mixdown from multiple stem files
pub fn generate_mixdown(
  song_id: &str,
  stem_file_paths: &[PathBuf],
) -> Result<String, ImportError> {
  if stem_file_paths.is_empty() {
    return Err(ImportError::Validation("No stem files provided for mixdown".to_string()));
  }

  log::info!("Generating mixdown for song {} from {} stems", song_id, stem_file_paths.len());

  // If only one file, just copy it as the mixdown
  if stem_file_paths.len() == 1 {
    let mixdowns_dir = get_mixdowns_directory()?;
    let mixdown_filename = get_mixdown_filename(song_id);
    let mixdown_path = mixdowns_dir.join(&mixdown_filename);

    // Simply copy the single file
    fs::copy(&stem_file_paths[0], &mixdown_path)?;

    log::info!("Single stem - copied to mixdown: {}", mixdown_path.display());
    return Ok(mixdown_path.to_string_lossy().to_string());
  }

  // Decode all stem files
  let mut decoded_stems = Vec::new();
  let mut target_sample_rate = 0u32;
  let mut max_length = 0usize;

  for file_path in stem_file_paths {
    log::info!("Decoding stem: {}", file_path.display());
    let (left, right, sample_rate) = decode_audio_file(file_path)?;

    if target_sample_rate == 0 {
      target_sample_rate = sample_rate;
    } else if target_sample_rate != sample_rate {
      log::warn!(
        "Sample rate mismatch: {} vs {}. Using {}",
        sample_rate,
        target_sample_rate,
        target_sample_rate
      );
    }

    max_length = max_length.max(left.len());
    decoded_stems.push((left, right));
  }

  // Mix all stems together
  let mut mixed_left = vec![0.0f32; max_length];
  let mut mixed_right = vec![0.0f32; max_length];

  for (left, right) in &decoded_stems {
    for (i, &sample) in left.iter().enumerate() {
      mixed_left[i] += sample;
    }
    for (i, &sample) in right.iter().enumerate() {
      mixed_right[i] += sample;
    }
  }

  // Normalize to prevent clipping
  let max_amplitude = mixed_left.iter()
    .chain(mixed_right.iter())
    .map(|&s| s.abs())
    .fold(0.0f32, f32::max);

  if max_amplitude > 1.0 {
    let scale = 1.0 / max_amplitude;
    for sample in &mut mixed_left {
      *sample *= scale;
    }
    for sample in &mut mixed_right {
      *sample *= scale;
    }
    log::info!("Normalized mixdown by factor of {}", scale);
  }

  // Write mixdown to WAV file
  let mixdowns_dir = get_mixdowns_directory()?;
  let mixdown_filename = get_mixdown_filename(song_id);
  let mixdown_path = mixdowns_dir.join(&mixdown_filename);

  let spec = WavSpec {
    channels: 2,
    sample_rate: target_sample_rate,
    bits_per_sample: 16,
    sample_format: hound::SampleFormat::Int,
  };

  let mut writer = WavWriter::create(&mixdown_path, spec)
    .map_err(|e| ImportError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

  // Interleave left and right channels and write
  for i in 0..max_length {
    let left_sample = (mixed_left[i] * 32767.0) as i16;
    let right_sample = (mixed_right[i] * 32767.0) as i16;

    writer.write_sample(left_sample)
      .map_err(|e| ImportError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
    writer.write_sample(right_sample)
      .map_err(|e| ImportError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
  }

  writer.finalize()
    .map_err(|e| ImportError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

  log::info!("Mixdown generated successfully: {}", mixdown_path.display());
  Ok(mixdown_path.to_string_lossy().to_string())
}
