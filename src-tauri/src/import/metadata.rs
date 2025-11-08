use std::fs::File;
use std::path::Path;
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use super::ImportError;

#[derive(Debug, Clone)]
pub struct AudioMetadata {
  pub sample_rate: i32,
  pub channels: i32,
  pub duration: f64,
  pub file_size: i64,
}

/// Extract metadata from an audio file using symphonia
pub fn extract_metadata(file_path: &Path) -> Result<AudioMetadata, ImportError> {
  // Check if file exists
  if !file_path.exists() {
    return Err(ImportError::FileNotFound(file_path.to_string_lossy().to_string()));
  }

  // Get file size
  let file_size = std::fs::metadata(file_path)
    .map_err(|e| ImportError::MetadataExtraction(format!("Failed to read file metadata: {}", e)))?
    .len() as i64;

  // Open the file
  let file = File::open(file_path)
    .map_err(|e| ImportError::MetadataExtraction(format!("Failed to open file: {}", e)))?;

  // Create media source stream
  let mss = MediaSourceStream::new(Box::new(file), Default::default());

  // Create a hint to help the format registry guess the format
  let mut hint = Hint::new();
  if let Some(extension) = file_path.extension() {
    hint.with_extension(&extension.to_string_lossy());
  }

  // Probe the media source
  let format_opts = FormatOptions::default();
  let metadata_opts = MetadataOptions::default();

  let probed = symphonia::default::get_probe()
    .format(&hint, mss, &format_opts, &metadata_opts)
    .map_err(|e| ImportError::InvalidFormat(format!("Failed to probe format: {}", e)))?;

  let mut format = probed.format;

  // Get the default track (usually the first audio track)
  let track = format
    .default_track()
    .ok_or_else(|| ImportError::InvalidFormat("No audio track found".to_string()))?;

  // Extract codec parameters
  let track_id = track.id;
  let codec_params = track.codec_params.clone();

  let sample_rate = codec_params
    .sample_rate
    .ok_or_else(|| ImportError::MetadataExtraction("No sample rate found".to_string()))? as i32;

  let channels = codec_params
    .channels
    .ok_or_else(|| ImportError::MetadataExtraction("No channel info found".to_string()))?
    .count() as i32;

  // Calculate duration
  let duration = if let Some(n_frames) = codec_params.n_frames {
    n_frames as f64 / sample_rate as f64
  } else {
    // If n_frames is not available, try to calculate from time_base and duration
    if let (Some(tb), Some(dur)) = (codec_params.time_base, codec_params.n_frames) {
      dur as f64 * tb.numer as f64 / tb.denom as f64
    } else {
      // Last resort: decode entire file to get duration (slower but accurate)
      calculate_duration_by_decoding(&mut format, track_id, &codec_params, sample_rate)?
    }
  };

  Ok(AudioMetadata {
    sample_rate,
    channels,
    duration,
    file_size,
  })
}

/// Calculate duration by decoding the entire audio stream (fallback method)
fn calculate_duration_by_decoding(
  format: &mut Box<dyn symphonia::core::formats::FormatReader>,
  track_id: u32,
  codec_params: &symphonia::core::codecs::CodecParameters,
  sample_rate: i32,
) -> Result<f64, ImportError> {
  // Create decoder
  let mut decoder = symphonia::default::get_codecs()
    .make(codec_params, &DecoderOptions::default())
    .map_err(|e| ImportError::MetadataExtraction(format!("Failed to create decoder: {}", e)))?;

  let mut total_frames: u64 = 0;

  // Decode all packets to count frames
  loop {
    match format.next_packet() {
      Ok(packet) => {
        // Skip packets not from our track
        if packet.track_id() != track_id {
          continue;
        }

        match decoder.decode(&packet) {
          Ok(decoded) => {
            total_frames += decoded.capacity() as u64;
          }
          Err(symphonia::core::errors::Error::DecodeError(_)) => {
            // Skip decode errors and continue
            continue;
          }
          Err(_) => break,
        }
      }
      Err(_) => break,
    }
  }

  let duration = total_frames as f64 / sample_rate as f64;
  Ok(duration)
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::path::PathBuf;

  #[test]
  fn test_extract_metadata_nonexistent_file() {
    let result = extract_metadata(&PathBuf::from("/nonexistent/file.wav"));
    assert!(result.is_err());
    match result {
      Err(ImportError::FileNotFound(_)) => (),
      _ => panic!("Expected FileNotFound error"),
    }
  }
}
