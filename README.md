# GoldSrc DSP

A Rust port of the **GoldSrc engine** reverb/DSP pipeline (`s_dsp.c`), providing the full effect chain used in Half-Life and other GoldSrc titles.

The crate ships as both a **library** (for embedding in a VST3/CLAP plugin via [nih-plug](https://github.com/robbert-vdh/nih-plug)) and a **standalone CLI binary** for offline WAV processing and algorithm validation.


## Building

Requires **Rust 1.70+** and Cargo.

### Library only

```bash
cargo build --lib
```

This produces `libgoldsrc_dsp` which can be consumed by another crate:

```toml
# In your plugin's Cargo.toml
[dependencies]
goldsrc_dsp = { git = "https://github.com/Shipi1/goldsrc_dsp.git", rev = "17d5655" }
# Or, if using a local path:
# goldsrc_dsp = { path = "../goldsrc_dsp" }
```

```rust
use goldsrc_dsp::{GoldSrcReverb, ClipMode, preset_for_room};
```

### Binary (offline CLI)

```bash
cargo build --release
```

The binary is written to `target/release/goldsrc_dsp` (`.exe` on Windows).

## CLI Usage

```
goldsrc_dsp <INPUT> [OPTIONS]
```

### Options

| Flag | Default | Description |
|---|---|---|
| `--room <N>` | `5` | Room preset index (0–28) |
| `--mix <F>` | `0.17` | Reverb dry/wet (0.0 = dry, 1.0 = wet) |
| `--delay-mix <F>` | `0.25` | Echo wet amount (0.0 = off, 1.0 = full) |
| `--clip <MODE>` | `hard` | Output clipping: `hard`, `soft` or `off` |
| `-o, --output <PATH>` | auto | Output WAV path (default: `<input>_<room>.wav`) |
| `--all` | — | Render all 29 room presets into a subfolder |

### Examples

```bash
# Process with room preset 5 (tunnel) at default mix
goldsrc_dsp drums.wav --room 5

# Fully wet reverb with soft clipping
goldsrc_dsp vocals.wav --room 7 --mix 1.0 --clip soft

# Custom output path
goldsrc_dsp snare.wav --room 14 --output wet_snare.wav

# Render every preset for comparison
goldsrc_dsp impulse.wav --all
```

### Room Presets

| ID | Name | ID | Name | ID | Name |
|---|---|---|---|---|---|
| 0 | off | 10 | chamber3 | 20 | outside |
| 1 | generic | 11 | bright | 21 | outside2 |
| 2 | metallic | 12 | bright2 | 22 | outside3 |
| 3 | metallic2 | 13 | bright3 | 23 | cavern |
| 4 | metallic3 | 14 | water | 24 | cavern2 |
| 5 | tunnel | 15 | water2 | 25 | cavern3 |
| 6 | tunnel2 | 16 | water3 | 26 | weirdo |
| 7 | tunnel3 | 17 | concrete | 27 | weirdo2 |
| 8 | chamber | 18 | concrete2 | 28 | weirdo3 |
| 9 | chamber2 | 19 | concrete3 | | |

## Library API

```rust
let mut reverb = GoldSrcReverb::new(sample_rate);
reverb.set_room_type(preset_for_room(5));
reverb.set_reverb_mix(0.17);   // dry/wet
reverb.set_delay_mix(0.25);    // echo level
reverb.set_clip_mode(ClipMode::Hard);

// Process a stereo buffer
reverb.process(&input_l, &input_r, &mut output_l, &mut output_r);
```
