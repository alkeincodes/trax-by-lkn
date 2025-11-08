pub struct LinearResampler {
  source_rate: u32,
  target_rate: u32,
  channels: u16,
  buffer: Vec<f32>,
  position: f64,
}

impl LinearResampler {
  pub fn new(source_rate: u32, target_rate: u32, channels: u16) -> Self {
    Self {
      source_rate,
      target_rate,
      channels,
      buffer: Vec::new(),
      position: 0.0,
    }
  }

  pub fn process(&mut self, input: &[f32]) -> Vec<f32> {
    if self.source_rate == self.target_rate {
      return input.to_vec();
    }

    let ratio = self.source_rate as f64 / self.target_rate as f64;
    let input_frames = input.len() / self.channels as usize;
    let output_frames = (input_frames as f64 / ratio).ceil() as usize;
    let mut output = vec![0.0; output_frames * self.channels as usize];

    for out_frame in 0..output_frames {
      let src_pos = out_frame as f64 * ratio;
      let src_idx = src_pos.floor() as usize;
      let frac = src_pos - src_idx as f64;

      if src_idx + 1 < input_frames {
        for ch in 0..self.channels as usize {
          let s0 = input[src_idx * self.channels as usize + ch];
          let s1 = input[(src_idx + 1) * self.channels as usize + ch];
          output[out_frame * self.channels as usize + ch] =
            s0 + (s1 - s0) * frac as f32;
        }
      } else if src_idx < input_frames {
        for ch in 0..self.channels as usize {
          output[out_frame * self.channels as usize + ch] =
            input[src_idx * self.channels as usize + ch];
        }
      }
    }

    output
  }

  pub fn reset(&mut self) {
    self.position = 0.0;
    self.buffer.clear();
  }
}
