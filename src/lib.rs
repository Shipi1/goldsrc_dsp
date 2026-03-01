//! GoldSrc Reverb DSP — Library crate
//!
//! Exposes the full DSP engine so it can be consumed by a nih_plug
//! (VST3/CLAP) plugin crate with a single dependency declaration:
//!
//! ```toml
//! # In your plugin's Cargo.toml:
//! [dependencies]
//! goldsrc_dsp = { path = "../goldsrc_dsp" }
//! ```
//!
//! Then in your plugin:
//! ```rust
//! use goldsrc_dsp::{GoldSrcReverb, ClipMode, Preset, preset_for_room};
//! ```

pub mod delay_line;
pub mod reverb;

// Flat re-exports at the crate root — the plugin only needs to import from
// here, not from the inner module paths.
pub use reverb::{
    preset_for_room, soft_clip_knee, ClipMode, GoldSrcReverb, Preset, PRESETS, ROOM_NAMES,
};
