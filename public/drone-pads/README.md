# Drone Pads Audio Files

This directory contains the drone pad audio files for TraX.

## Structure

Each preset should be organized in its own folder with 12 audio files named after musical keys:

```
drone-pads/
├── warm-pads-by-churchfront/
│   ├── C.mp3
│   ├── Db.mp3
│   ├── D.mp3
│   ├── Eb.mp3
│   ├── E.mp3
│   ├── F.mp3
│   ├── Gb.mp3
│   ├── G.mp3
│   ├── Ab.mp3
│   ├── A.mp3
│   ├── Bb.mp3
│   └── B.mp3
└── ambient-pads-by-vishal-bhojane/
    ├── C.mp3
    ├── Db.mp3
    ├── D.mp3
    ├── Eb.mp3
    ├── E.mp3
    ├── F.mp3
    ├── Gb.mp3
    ├── G.mp3
    ├── Ab.mp3
    ├── A.mp3
    ├── Bb.mp3
    └── B.mp3
```

## Adding New Presets

1. Create a new folder in `drone-pads/` with a kebab-case name
2. Add all 12 key files (C.mp3, Db.mp3, D.mp3, Eb.mp3, E.mp3, F.mp3, Gb.mp3, G.mp3, Ab.mp3, A.mp3, Bb.mp3, B.mp3)
3. Update the `dronePresets` array in `src/stores/dronePad.ts` to include the new preset

## Audio File Requirements

- Format: MP3
- Sample Rate: 44.1kHz or 48kHz recommended
- Bit Rate: 320kbps recommended
- Duration: 30 seconds minimum (files will loop)
- Fade: Include natural loop points or crossfade-friendly endings
