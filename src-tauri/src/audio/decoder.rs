use super::types::{AudioError, AudioMetadata, AudioResult};
use std::fs::File;
use std::path::Path;
use symphonia::core::audio::{AudioBufferRef, Signal};
use symphonia::core::codecs::{Decoder, DecoderOptions, CODEC_TYPE_NULL};
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::{FormatOptions, FormatReader};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

pub struct AudioDecoder {
  format: Box<dyn FormatReader>,
  decoder: Box<dyn Decoder>,
  track_id: u32,
}

impl AudioDecoder {
  pub fn new(path: &str) -> AudioResult<Self> {
    let src = File::open(path).map_err(|e| AudioError::FileError(e.to_string()))?;

    let mss = MediaSourceStream::new(Box::new(src), Default::default());

    let mut hint = Hint::new();
    if let Some(ext) = Path::new(path).extension().and_then(|e| e.to_str()) {
      hint.with_extension(ext);
    }

    let meta_opts: MetadataOptions = Default::default();
    let fmt_opts: FormatOptions = Default::default();

    let probed = symphonia::default::get_probe()
      .format(&hint, mss, &fmt_opts, &meta_opts)
      .map_err(|e| AudioError::DecodeError(format!("Failed to probe file: {}", e)))?;

    let format = probed.format;

    let track = format
      .tracks()
      .iter()
      .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
      .ok_or_else(|| AudioError::InvalidFormat("No supported audio track found".to_string()))?;

    let track_id = track.id;

    let dec_opts: DecoderOptions = Default::default();
    let decoder = symphonia::default::get_codecs()
      .make(&track.codec_params, &dec_opts)
      .map_err(|e| AudioError::DecodeError(format!("Failed to create decoder: {}", e)))?;

    Ok(Self {
      format,
      decoder,
      track_id,
    })
  }

  pub fn get_metadata(&self) -> AudioResult<AudioMetadata> {
    let track = self
      .format
      .tracks()
      .iter()
      .find(|t| t.id == self.track_id)
      .ok_or_else(|| AudioError::InvalidFormat("Track not found".to_string()))?;

    let codec_params = &track.codec_params;

    let sample_rate = codec_params
      .sample_rate
      .ok_or_else(|| AudioError::InvalidFormat("Sample rate not available".to_string()))?;

    let channels = codec_params
      .channels
      .map(|c| c.count() as u16)
      .ok_or_else(|| AudioError::InvalidFormat("Channel count not available".to_string()))?;

    let duration = if let Some(n_frames) = codec_params.n_frames {
      n_frames as f64 / sample_rate as f64
    } else {
      0.0
    };

    let format = codec_params
      .codec
      .to_string()
      .split_whitespace()
      .next()
      .unwrap_or("unknown")
      .to_uppercase();

    Ok(AudioMetadata {
      duration,
      sample_rate,
      channels,
      format,
    })
  }

  pub fn decode_next_packet(&mut self) -> AudioResult<Option<DecodedAudio>> {
    loop {
      let packet = match self.format.next_packet() {
        Ok(packet) => packet,
        Err(SymphoniaError::ResetRequired) => {
          return Err(AudioError::DecodeError("Decoder reset required".to_string()));
        }
        Err(SymphoniaError::IoError(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
          return Ok(None);
        }
        Err(e) => {
          return Err(AudioError::DecodeError(format!("Failed to read packet: {}", e)));
        }
      };

      if packet.track_id() != self.track_id {
        continue;
      }

      match self.decoder.decode(&packet) {
        Ok(decoded) => {
          let samples = convert_audio_buffer(decoded)?;
          return Ok(Some(DecodedAudio { samples }));
        }
        Err(SymphoniaError::DecodeError(e)) => {
          log::warn!("Decode error: {}, skipping packet", e);
          continue;
        }
        Err(e) => {
          return Err(AudioError::DecodeError(format!("Decoder error: {}", e)));
        }
      }
    }
  }

  pub fn seek(&mut self, time_seconds: f64) -> AudioResult<()> {
    let track = self
      .format
      .tracks()
      .iter()
      .find(|t| t.id == self.track_id)
      .ok_or_else(|| AudioError::InvalidFormat("Track not found".to_string()))?;

    let sample_rate = track
      .codec_params
      .sample_rate
      .ok_or_else(|| AudioError::InvalidFormat("Sample rate not available".to_string()))?;

    let target_sample = (time_seconds * sample_rate as f64) as u64;

    self
      .format
      .seek(
        symphonia::core::formats::SeekMode::Accurate,
        symphonia::core::formats::SeekTo::TimeStamp {
          ts: target_sample,
          track_id: self.track_id,
        },
      )
      .map_err(|e| AudioError::PlaybackError(format!("Seek failed: {}", e)))?;

    self.decoder.reset();

    Ok(())
  }
}

pub struct DecodedAudio {
  pub samples: Vec<f32>,
}

fn convert_audio_buffer(buffer: AudioBufferRef) -> AudioResult<Vec<f32>> {
  match buffer {
    AudioBufferRef::F32(buf) => {
      let num_channels = buf.spec().channels.count();
      let mut samples = Vec::with_capacity(buf.frames() * num_channels);
      for frame_idx in 0..buf.frames() {
        for channel_idx in 0..num_channels {
          samples.push(buf.chan(channel_idx)[frame_idx]);
        }
      }
      Ok(samples)
    }
    AudioBufferRef::S16(buf) => {
      let num_channels = buf.spec().channels.count();
      let mut samples = Vec::with_capacity(buf.frames() * num_channels);
      for frame_idx in 0..buf.frames() {
        for channel_idx in 0..num_channels {
          let sample_i16 = buf.chan(channel_idx)[frame_idx];
          let sample_f32 = sample_i16 as f32 / i16::MAX as f32;
          samples.push(sample_f32);
        }
      }
      Ok(samples)
    }
    AudioBufferRef::S24(buf) => {
      let num_channels = buf.spec().channels.count();
      let mut samples = Vec::with_capacity(buf.frames() * num_channels);
      for frame_idx in 0..buf.frames() {
        for channel_idx in 0..num_channels {
          let sample_i24 = buf.chan(channel_idx)[frame_idx];
          let sample_i32 = sample_i24.inner();
          let sample_f32 = sample_i32 as f32 / 8388608.0;
          samples.push(sample_f32);
        }
      }
      Ok(samples)
    }
    AudioBufferRef::S32(buf) => {
      let num_channels = buf.spec().channels.count();
      let mut samples = Vec::with_capacity(buf.frames() * num_channels);
      for frame_idx in 0..buf.frames() {
        for channel_idx in 0..num_channels {
          let sample_i32 = buf.chan(channel_idx)[frame_idx];
          let sample_f32 = sample_i32 as f32 / i32::MAX as f32;
          samples.push(sample_f32);
        }
      }
      Ok(samples)
    }
    _ => Err(AudioError::DecodeError(
      "Unsupported audio buffer format".to_string(),
    )),
  }
}
