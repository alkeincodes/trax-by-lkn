use std::path::Path;

/// Detect stem name from filename using common keywords
pub fn detect_stem_name(filename: &str) -> String {
  // Remove file extension
  let name_without_ext = Path::new(filename)
    .file_stem()
    .and_then(|s| s.to_str())
    .unwrap_or(filename);

  // Convert to lowercase for case-insensitive matching
  let lowercase = name_without_ext.to_lowercase();

  // Common stem keywords (in order of priority)
  let keywords = vec![
    ("vocals", "Vocals"),
    ("vox", "Vox"),
    ("drums", "Drums"),
    ("bass", "Bass"),
    ("keys", "Keys"),
    ("keyboard", "Keyboard"),
    ("piano", "Piano"),
    ("guitar", "Guitar"),
    ("synth", "Synth"),
    ("pad", "Pad"),
    ("strings", "Strings"),
    ("orchestra", "Orchestra"),
    ("click", "Click"),
    ("guide", "Guide"),
  ];

  // Try to extract stem name from various patterns

  // Pattern 1: "Song Name - Vocals.wav" or "Song Name - Vocals 01.wav"
  if let Some(after_dash) = lowercase.split(" - ").nth(1) {
    for (keyword, display) in &keywords {
      if after_dash.contains(keyword) {
        return display.to_string();
      }
    }
  }

  // Pattern 2: "Song Name_Vocals.wav"
  if let Some(after_underscore) = lowercase.split('_').last() {
    for (keyword, display) in &keywords {
      if after_underscore.contains(keyword) {
        return display.to_string();
      }
    }
  }

  // Pattern 3: "Song Name (Vocals).wav"
  if let Some(start) = lowercase.find('(') {
    if let Some(end) = lowercase.find(')') {
      if end > start {
        let in_parens = &lowercase[start + 1..end];
        for (keyword, display) in &keywords {
          if in_parens.contains(keyword) {
            return display.to_string();
          }
        }
      }
    }
  }

  // Pattern 4: Simple keyword match in entire filename
  for (keyword, display) in &keywords {
    if lowercase.contains(keyword) {
      return display.to_string();
    }
  }

  // Fallback: Use filename without extension, cleaned up
  clean_filename(name_without_ext)
}

/// Clean up filename by removing common patterns
fn clean_filename(name: &str) -> String {
  let mut result = name.to_string();

  // Remove numbers and underscores at the end
  result = result.trim_end_matches(|c: char| c.is_numeric() || c == '_' || c == ' ').to_string();

  // If result is empty or too short, use original
  if result.is_empty() || result.len() < 2 {
    result = name.to_string();
  }

  // Capitalize first letter
  let mut chars = result.chars();
  match chars.next() {
    None => String::new(),
    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_detect_stem_name_basic() {
    assert_eq!(detect_stem_name("vocals.wav"), "Vocals");
    assert_eq!(detect_stem_name("drums.mp3"), "Drums");
    assert_eq!(detect_stem_name("bass.flac"), "Bass");
  }

  #[test]
  fn test_detect_stem_name_with_dash() {
    assert_eq!(detect_stem_name("Song Name - Vocals.wav"), "Vocals");
    assert_eq!(detect_stem_name("Amazing Track - Drums.mp3"), "Drums");
  }

  #[test]
  fn test_detect_stem_name_with_underscore() {
    assert_eq!(detect_stem_name("song_vocals.wav"), "Vocals");
    assert_eq!(detect_stem_name("track_drums.mp3"), "Drums");
  }

  #[test]
  fn test_detect_stem_name_with_parentheses() {
    assert_eq!(detect_stem_name("Song (Vocals).wav"), "Vocals");
    assert_eq!(detect_stem_name("Track (Drums).mp3"), "Drums");
  }

  #[test]
  fn test_detect_stem_name_case_insensitive() {
    assert_eq!(detect_stem_name("VOCALS.wav"), "Vocals");
    assert_eq!(detect_stem_name("DrUmS.mp3"), "Drums");
  }

  #[test]
  fn test_detect_stem_name_all_keywords() {
    let test_cases = vec![
      ("vocals.wav", "Vocals"),
      ("vox.wav", "Vox"),
      ("drums.wav", "Drums"),
      ("bass.wav", "Bass"),
      ("keys.wav", "Keys"),
      ("keyboard.wav", "Keyboard"),
      ("piano.wav", "Piano"),
      ("guitar.wav", "Guitar"),
      ("synth.wav", "Synth"),
      ("pad.wav", "Pad"),
      ("strings.wav", "Strings"),
      ("orchestra.wav", "Orchestra"),
      ("click.wav", "Click"),
      ("guide.wav", "Guide"),
    ];

    for (input, expected) in test_cases {
      assert_eq!(detect_stem_name(input), expected);
    }
  }

  #[test]
  fn test_clean_filename() {
    assert_eq!(clean_filename("vocals_01"), "Vocals");
    assert_eq!(clean_filename("drums_02_"), "Drums");
    assert_eq!(clean_filename("custom_name"), "Custom_name");
  }
}
