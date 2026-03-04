use rand::rngs::StdRng;
/// GoldSrc Reverb DSP — Rust Port (f32-based)
///
/// Direct port of `s_dsp.c` for offline algorithm validation.
use rand::{Rng, SeedableRng};

use crate::delay_line::DelayLine;

/// Fixed RNG seed — guarantees identical modulation behaviour every run.
const RNG_SEED: u64 = 42;

// =============================================================================
//  Constants
// =============================================================================

const REVERB_XFADE: i32 = 32;
const STEREO_XFADE: i32 = 128;
const MAX_REVERB_DELAY: f32 = 0.1;
const MAX_MONO_DELAY: f32 = 0.4;
const MAX_STEREO_DELAY: f32 = 0.1;
const SOFT_CLIP_THRESHOLD: f32 = 0.8;

// =============================================================================
//  Preset Table (rgsxpre[])
// =============================================================================

/// One preset: [lp, mod, size, refl, rvblp, delay, feedback, dlylp, left]
pub type Preset = [f32; 9];

const P_LP: usize = 0;
const P_MOD: usize = 1;
const P_SIZE: usize = 2;
const P_REFL: usize = 3;
const P_RVBLP: usize = 4;
const P_DELAY: usize = 5;
const P_FEEDBACK: usize = 6;
const P_DLYLP: usize = 7;
const P_LEFT: usize = 8;

pub const PRESETS: [Preset; 29] = [
    // [lp,  mod,  size,  refl,  rvblp, delay,  feedback, dlylp, left]
    [0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 2.0, 0.0], //  0 off
    [0.0, 0.0, 0.0, 0.0, 1.0, 0.065, 0.1, 0.0, 0.01], //  1 generic
    [0.0, 0.0, 0.0, 0.0, 1.0, 0.02, 0.75, 0.0, 0.01], //  2 metallic
    [0.0, 0.0, 0.0, 0.0, 1.0, 0.03, 0.78, 0.0, 0.02], //  3
    [0.0, 0.0, 0.0, 0.0, 1.0, 0.06, 0.77, 0.0, 0.03], //  4
    [0.0, 0.0, 0.05, 0.85, 1.0, 0.008, 0.2, 2.0, 0.01], //  5 tunnel
    [0.0, 0.0, 0.05, 0.88, 1.0, 0.01, 0.98, 2.0, 0.02], //  6
    [0.0, 0.0, 0.05, 0.92, 1.0, 0.015, 0.995, 2.0, 0.04], //  7
    [0.0, 0.0, 0.05, 0.84, 1.0, 0.0, 0.0, 2.0, 0.012], //  8 chamber
    [0.0, 0.0, 0.05, 0.9, 1.0, 0.0, 0.0, 2.0, 0.008], //  9
    [0.0, 0.0, 0.05, 0.95, 1.0, 0.0, 0.0, 2.0, 0.004], // 10
    [0.0, 0.0, 0.05, 0.7, 0.0, 0.0, 0.0, 2.0, 0.012], // 11 bright
    [0.0, 0.0, 0.055, 0.78, 0.0, 0.0, 0.0, 2.0, 0.008], // 12
    [0.0, 0.0, 0.05, 0.86, 0.0, 0.0, 0.0, 2.0, 0.002], // 13
    [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 2.0, 0.01], // 14 water
    [1.0, 0.0, 0.0, 0.0, 1.0, 0.06, 0.85, 2.0, 0.02], // 15
    [1.0, 0.0, 0.0, 0.0, 1.0, 0.2, 0.6, 2.0, 0.05], // 16
    [0.0, 0.0, 0.05, 0.8, 1.0, 0.0, 0.48, 2.0, 0.016], // 17 concrete
    [0.0, 0.0, 0.06, 0.9, 1.0, 0.0, 0.52, 2.0, 0.01], // 18
    [0.0, 0.0, 0.07, 0.94, 1.0, 0.3, 0.6, 2.0, 0.008], // 19
    [0.0, 0.0, 0.0, 0.0, 1.0, 0.3, 0.42, 2.0, 0.0], // 20 outside
    [0.0, 0.0, 0.0, 0.0, 1.0, 0.35, 0.48, 2.0, 0.0], // 21
    [0.0, 0.0, 0.0, 0.0, 1.0, 0.38, 0.6, 2.0, 0.0], // 22
    [0.0, 0.0, 0.05, 0.9, 1.0, 0.2, 0.28, 0.0, 0.0], // 23 cavern
    [0.0, 0.0, 0.07, 0.9, 1.0, 0.3, 0.4, 0.0, 0.0], // 24
    [0.0, 0.0, 0.09, 0.9, 1.0, 0.35, 0.5, 0.0, 0.0], // 25
    [0.0, 1.0, 0.01, 0.9, 0.0, 0.0, 0.0, 2.0, 0.05], // 26 weirdo
    [0.0, 0.0, 0.0, 0.0, 1.0, 0.009, 0.999, 2.0, 0.04], // 27
    [0.0, 0.0, 0.001, 0.999, 0.0, 0.2, 0.8, 2.0, 0.05], // 28
];

pub const ROOM_NAMES: [&str; 29] = [
    "off",
    "generic",
    "metallic",
    "metallic2",
    "metallic3",
    "tunnel",
    "tunnel2",
    "tunnel3",
    "chamber",
    "chamber2",
    "chamber3",
    "bright",
    "bright2",
    "bright3",
    "water",
    "water2",
    "water3",
    "concrete",
    "concrete2",
    "concrete3",
    "outside",
    "outside2",
    "outside3",
    "cavern",
    "cavern2",
    "cavern3",
    "weirdo",
    "weirdo2",
    "weirdo3",
];

/// Look up a preset by room index (0–28).
///
/// Use this to build the `Preset` array that `set_room_type()` expects:
/// ```
/// reverb.set_room_type(preset_for_room(5));
/// ```
pub fn preset_for_room(room: usize) -> Preset {
    PRESETS[room.min(PRESETS.len() - 1)]
}

// =============================================================================
//  Clip Modes
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipMode {
    Hard,
    Soft,
}

/// Soft clipper: tanh saturation above threshold (0.8).
#[inline]
pub fn soft_clip_knee(x: f32, threshold: f32) -> f32 {
    let headroom = 1.0 - threshold;
    let absx = x.abs();
    if absx <= threshold {
        x
    } else {
        let sign = x.signum();
        let excess = absx - threshold;
        sign * (threshold + headroom * (excess / headroom).tanh())
    }
}

// =============================================================================
//  GoldSrcReverb
// =============================================================================

pub struct GoldSrcReverb {
    pub sample_rate: u32,

    // Reverb delay lines (2 taps)
    reverb_dly: [DelayLine; 2],

    // Mono echo delay
    mono_dly: DelayLine,

    // Stereo widening delay
    stereo_dly: DelayLine,

    // Amplitude modulation state
    amod_l: f32,
    amod_r: f32,
    amod_lt: f32,
    amod_rt: f32,
    amod1_cur: i32,
    amod2_cur: i32,
    amod1: i32,
    amod2: i32,

    // Lowpass history (for do_amod)
    lp_history: [f32; 10],

    // Current preset values
    preset_lp: f32,
    preset_mod: f32,
    preset_rvb_lp: f32,

    // Active flags
    reverb_active: bool,
    mono_active: bool,
    stereo_active: bool,

    // RNG (seeded from RNG_SEED — deterministic)
    rng: StdRng,

    // Last applied preset array — None means no preset loaded yet.
    // Guards against redundant reconfiguration on every audio buffer.
    current_params: Option<Preset>,

    // Mix parameters — set via setters, read in process().
    reverb_mix: f32, // 0.0 = dry, 1.0 = fully wet
    delay_mix: f32,  // 0.0 = off, 1.0 = full echo
    clip_mode: ClipMode,
}

#[allow(dead_code)]
impl GoldSrcReverb {
    pub fn new(sample_rate: u32) -> Self {
        let sr = sample_rate;
        GoldSrcReverb {
            sample_rate: sr,
            reverb_dly: [
                DelayLine::new(MAX_REVERB_DELAY, sr),
                DelayLine::new(MAX_REVERB_DELAY, sr),
            ],
            mono_dly: DelayLine::new(MAX_MONO_DELAY, sr),
            stereo_dly: DelayLine::new(MAX_STEREO_DELAY, sr),
            amod_l: 1.0,
            amod_r: 1.0,
            amod_lt: 1.0,
            amod_rt: 1.0,
            amod1_cur: 0,
            amod2_cur: 0,
            amod1: (350.0_f32 * sr as f32 / 11025.0) as i32,
            amod2: (450.0_f32 * sr as f32 / 11025.0) as i32,
            lp_history: [0.0; 10],
            preset_lp: 0.0,
            preset_mod: 0.0,
            preset_rvb_lp: 1.0,
            reverb_active: false,
            mono_active: false,
            stereo_active: false,
            rng: StdRng::seed_from_u64(RNG_SEED),
            current_params: None,
            reverb_mix: 0.17, // GoldSrc default
            delay_mix: 0.25,  // GoldSrc default
            clip_mode: ClipMode::Hard,
        }
    }

    // -------------------------------------------------------------------------
    //  Preset Management
    // -------------------------------------------------------------------------

    /// Apply a preset parameter array.
    ///
    /// Build the array with `preset_for_room(n)` or supply custom values.
    ///
    /// **Safe to call every audio buffer** — does nothing when called with
    /// the same values as last time, so the plugin can pass the parameter
    /// directly every callback without tracking previous state.
    ///
    /// Only reconfigures delay lines when the values actually change,
    /// preserving reverb tails across the transition (no click).
    pub fn set_room_type(&mut self, params: Preset) {
        // No-op if the preset hasn't changed.
        if self.current_params == Some(params) {
            return;
        }
        self.current_params = Some(params);

        self.preset_lp = params[P_LP];
        self.preset_mod = params[P_MOD];
        self.preset_rvb_lp = params[P_RVBLP];

        self.setup_reverb(params[P_SIZE], params[P_REFL], params[P_RVBLP]);
        self.setup_mono_delay(params[P_DELAY], params[P_FEEDBACK], params[P_DLYLP]);
        self.setup_stereo_delay(params[P_LEFT]);
    }

    /// Reverb dry/wet mix. `0.0` = fully dry, `1.0` = fully wet.
    /// GoldSrc default: `0.17`.
    pub fn set_reverb_mix(&mut self, mix: f32) {
        self.reverb_mix = mix.clamp(0.0, 1.0);
    }

    /// Echo (mono delay) wet level. `0.0` = off, `1.0` = full.
    /// GoldSrc default: `0.25`.
    pub fn set_delay_mix(&mut self, mix: f32) {
        self.delay_mix = mix.clamp(0.0, 1.0);
    }

    /// Output clip mode. `Hard` clamps to ±1.0; `Soft` applies tanh saturation above 0.8.
    pub fn set_clip_mode(&mut self, mode: ClipMode) {
        self.clip_mode = mode;
    }

    /// Re-seed the internal RNG. Takes effect immediately.
    pub fn set_rng_seed(&mut self, seed: u64) {
        self.rng = StdRng::seed_from_u64(seed);
    }

    /// Hard-reset all delay buffers and filter state to silence.
    ///
    /// Call this from the host's `reset()` / `initialize()` callback —
    /// **not** on a preset change. Resetting on preset change would kill
    /// the reverb tail and cause an audible click.
    pub fn reset_buffers(&mut self) {
        for dly in self.reverb_dly.iter_mut() {
            dly.reset();
            dly.input_pos = 0;
            dly.output_pos = 0;
        }
        self.mono_dly.reset();
        self.mono_dly.input_pos = 0;
        self.mono_dly.output_pos = 0;
        self.stereo_dly.reset();
        self.stereo_dly.input_pos = 0;
        self.stereo_dly.output_pos = 0;
        self.lp_history = [0.0; 10];
        self.amod_l = 1.0;
        self.amod_r = 1.0;
        self.amod_lt = 1.0;
        self.amod_rt = 1.0;
        // Force set_room_type() to re-apply the preset on next call.
        self.current_params = None;
    }

    fn setup_reverb(&mut self, size: f32, refl: f32, rvblp: f32) {
        if size == 0.0 {
            self.reverb_active = false;
            return;
        }

        self.reverb_active = true;
        let size = size.min(MAX_REVERB_DELAY);
        let samples = (size * self.sample_rate as f32) as usize;

        // Tap 0: full size, mod period ~500 samples at 11kHz scaled
        Self::init_reverb_tap(
            &mut self.reverb_dly[0],
            samples,
            refl,
            rvblp,
            500,
            self.sample_rate,
        );

        // Tap 1: 0.71x size, mod period ~700 samples at 11kHz scaled
        let samples2 = (samples as f32 * 0.71) as usize;
        Self::init_reverb_tap(
            &mut self.reverb_dly[1],
            samples2,
            refl,
            rvblp,
            700,
            self.sample_rate,
        );
    }

    fn init_reverb_tap(
        dly: &mut DelayLine,
        delay_samples: usize,
        feedback: f32,
        lp: f32,
        kmod: i32,
        sample_rate: u32,
    ) {
        // Do NOT reset the buffer — existing reverb tail decays naturally
        // under the new feedback value, avoiding a hard click on preset change.
        dly.delay_samples = delay_samples;
        dly.feedback = feedback;
        dly.lp_enabled = lp >= 1.0;
        dly.modulation = (kmod as f32 * sample_rate as f32 / 11025.0) as i32;
        dly.mod_cur = dly.modulation;
        // Reposition output_pos relative to the live write head.
        if delay_samples <= dly.buffer_size {
            dly.output_pos = (dly.input_pos + dly.buffer_size - delay_samples) % dly.buffer_size;
        } else {
            dly.output_pos = dly.input_pos;
        }
        dly.xfade = 0;
    }

    fn setup_mono_delay(&mut self, delay: f32, feedback: f32, dlylp: f32) {
        if delay == 0.0 {
            self.mono_active = false;
            return;
        }

        self.mono_active = true;
        let delay = delay.min(MAX_MONO_DELAY);
        let dly = &mut self.mono_dly;
        // Do NOT reset — let the echo tail decay under the new feedback value.
        dly.delay_samples = (delay * self.sample_rate as f32) as usize;
        dly.feedback = feedback;
        dly.lp_enabled = dlylp < 1.0;
        if dly.delay_samples <= dly.buffer_size {
            dly.output_pos =
                (dly.input_pos + dly.buffer_size - dly.delay_samples) % dly.buffer_size;
        } else {
            dly.output_pos = dly.input_pos;
        }
    }

    fn setup_stereo_delay(&mut self, left_delay: f32) {
        if left_delay == 0.0 {
            self.stereo_active = false;
            return;
        }

        self.stereo_active = true;
        let left_delay = left_delay.min(MAX_STEREO_DELAY);
        let dly = &mut self.stereo_dly;
        // Do NOT reset — reposition read head smoothly relative to write head.
        dly.delay_samples = (left_delay * self.sample_rate as f32) as usize;
        dly.modulation = 0;
        dly.mod_cur = 0;
        dly.xfade = 0;
        if dly.delay_samples <= dly.buffer_size {
            dly.output_pos =
                (dly.input_pos + dly.buffer_size - dly.delay_samples) % dly.buffer_size;
        } else {
            dly.output_pos = dly.input_pos;
        }
    }

    // -------------------------------------------------------------------------
    //  Core Processing
    // -------------------------------------------------------------------------

    /// Process a stereo buffer through the full DSP chain **in-place**.
    ///
    /// **Zero heap allocations** — suitable for real-time audio callbacks.
    /// `out_l` and `out_r` must be the same length as `left`/`right`.
    /// On entry their contents are overwritten; on exit they hold the result.
    ///
    /// Mix parameters are read from internal state — set them with
    /// `set_reverb_mix()`, `set_delay_mix()`, and `set_clip_mode()`.
    ///
    /// Signal flow:
    ///   Input → AMod → DoReverb (100% wet) → DoDelay (additive) → DoStereoDelay → Dry/Wet → Output
    pub fn process(&mut self, left: &[f32], right: &[f32], out_l: &mut [f32], out_r: &mut [f32]) {
        let n = left.len();

        // Debug-only validation; in release, silently bail on bad buffers
        // to avoid panicking across the C FFI boundary (UB in VST/CLAP hosts).
        debug_assert_eq!(n, right.len(), "L/R input buffers must be equal length");
        debug_assert!(out_l.len() >= n, "out_l too small");
        debug_assert!(out_r.len() >= n, "out_r too small");

        if right.len() != n || out_l.len() < n || out_r.len() < n {
            return;
        }

        let reverb_mix = self.reverb_mix;
        let delay_mix = self.delay_mix;
        let clip_mode = self.clip_mode;

        // Copy input → output (working buffers)
        out_l[..n].copy_from_slice(left);
        out_r[..n].copy_from_slice(right);

        // Step 1: Amplitude Modulation & Lowpass (underwater)
        if self.preset_lp != 0.0 || self.preset_mod != 0.0 {
            self.do_amod(&mut out_l[..n], &mut out_r[..n]);
        }

        // Step 2: Reverb (100% wet output)
        if self.reverb_active {
            self.do_reverb_inplace(&mut out_l[..n], &mut out_r[..n]);
        }

        // Step 3: Mono Echo (additive on top of mixed signal)
        if self.mono_active {
            self.do_delay(&mut out_l[..n], &mut out_r[..n], delay_mix);
        }

        // Step 4: Stereo Widening (Haas effect)
        if self.stereo_active {
            self.do_stereo_delay(&mut out_l[..n], &mut out_r[..n]);
        }

        // Step 5: Final dry/wet crossfade (applied once, after the full wet chain)
        {
            let dry_mix = 1.0 - reverb_mix;
            for i in 0..n {
                out_l[i] = dry_mix * left[i] + reverb_mix * out_l[i];
                out_r[i] = dry_mix * right[i] + reverb_mix * out_r[i];
            }
        }

        // Final output limiter
        match clip_mode {
            ClipMode::Soft => {
                for v in out_l.iter_mut() {
                    *v = soft_clip_knee(*v, SOFT_CLIP_THRESHOLD);
                }
                for v in out_r.iter_mut() {
                    *v = soft_clip_knee(*v, SOFT_CLIP_THRESHOLD);
                }
            }
            ClipMode::Hard => {
                for v in out_l.iter_mut() {
                    *v = v.clamp(-1.0, 1.0);
                }
                for v in out_r.iter_mut() {
                    *v = v.clamp(-1.0, 1.0);
                }
            }
        }
    }

    // -------------------------------------------------------------------------
    //  RVB_DoAMod — Amplitude Modulation + Lowpass
    // -------------------------------------------------------------------------

    #[inline]
    fn do_amod(&mut self, left: &mut [f32], right: &mut [f32]) {
        let n = left.len();

        for i in 0..n {
            let mut res_l = left[i];
            let mut res_r = right[i];

            // Lowpass (6-tap moving average)
            if self.preset_lp != 0.0 {
                let lp = &self.lp_history;
                res_l = (lp[0] + lp[1] + lp[2] + lp[3] + lp[4] + res_l) / 4.0;
                res_r = (lp[5] + lp[6] + lp[7] + lp[8] + lp[9] + res_r) / 4.0;

                self.lp_history[4] = left[i];
                self.lp_history[9] = right[i];
                // Shift left
                self.lp_history.copy_within(1..5, 0);
                self.lp_history.copy_within(6..10, 5);
            }

            // Amplitude modulation (tremolo)
            if self.preset_mod != 0.0 {
                self.amod1_cur -= 1;
                if self.amod1_cur < 0 {
                    self.amod1_cur = self.amod1;
                }
                if self.amod1_cur == 0 {
                    self.amod_lt = self.rng.gen_range(0.125_f32..=1.0_f32);
                }

                self.amod2_cur -= 1;
                if self.amod2_cur < 0 {
                    self.amod2_cur = self.amod2;
                }
                if self.amod2_cur == 0 {
                    self.amod_rt = self.rng.gen_range(0.125_f32..=1.0_f32);
                }

                res_l *= self.amod_l;
                res_r *= self.amod_r;

                // Smooth towards target
                let step = 1.0 / 255.0;
                if self.amod_l < self.amod_lt {
                    self.amod_l = (self.amod_l + step).min(self.amod_lt);
                } else if self.amod_l > self.amod_lt {
                    self.amod_l = (self.amod_l - step).max(self.amod_lt);
                }

                if self.amod_r < self.amod_rt {
                    self.amod_r = (self.amod_r + step).min(self.amod_rt);
                } else if self.amod_r > self.amod_rt {
                    self.amod_r = (self.amod_r - step).max(self.amod_rt);
                }
            }

            left[i] = res_l.clamp(-1.0, 1.0);
            right[i] = res_r.clamp(-1.0, 1.0);
        }
    }

    // -------------------------------------------------------------------------
    //  RVB_DoReverb — 2-tap feedback delay network
    // -------------------------------------------------------------------------

    /// In-place reverb: writes 100% wet reverb signal into `left`/`right`.
    /// The dry/wet crossfade is handled externally by `process()`.
    /// **Zero allocations.**
    #[inline]
    fn do_reverb_inplace(&mut self, left: &mut [f32], right: &mut [f32]) {
        let n = left.len();

        if self.reverb_dly[0].delay_samples == 0 {
            return;
        }

        // Pre-compute input scale per tap: 1 / (1 + feedback)
        let scale0 = 1.0 / (1.0 + self.reverb_dly[0].feedback);
        let scale1 = 1.0 / (1.0 + self.reverb_dly[1].feedback);

        for i in 0..n {
            let dry_l = left[i];
            let dry_r = right[i];

            // Mono downmix
            let vlr = (dry_l + dry_r) * 0.5;

            let wet0 = self.reverb_one_tap(0, vlr * scale0, dry_l, dry_r);
            let wet1 = self.reverb_one_tap(1, vlr * scale1, dry_l, dry_r);
            let wet = wet0 + wet1;

            // 100% wet — crossfade is applied later in process()
            left[i] = wet;
            right[i] = wet;
        }
    }

    #[inline]
    fn reverb_one_tap(&mut self, tap_idx: usize, vlr: f32, sample_l: f32, sample_r: f32) -> f32 {
        let dly = &mut self.reverb_dly[tap_idx];

        // Modulation timer
        dly.mod_cur -= 1;
        if dly.mod_cur < 0 {
            dly.mod_cur = dly.modulation;
        }

        // Read delayed sample
        let mut delay = dly.buffer[dly.output_pos];

        let voutm;
        if dly.xfade != 0 || delay.abs() > 1e-10 || sample_l.abs() > 1e-10 || sample_r.abs() > 1e-10
        {
            // Trigger modulation wobble
            if dly.mod_cur == 0 && dly.modulation > 0 {
                let rand_val: u32 = self.rng.gen_range(0..=255);
                let offset =
                    ((rand_val as f32 * delay.abs() * 512.0) as usize) % dly.delay_samples.max(1);
                dly.output_pos_xf = (dly.output_pos + offset) % dly.buffer_size;
                dly.xfade = REVERB_XFADE;
            }

            // Crossfade
            if dly.xfade != 0 {
                let sample_xf = dly.buffer[dly.output_pos_xf] * (REVERB_XFADE - dly.xfade) as f32
                    / REVERB_XFADE as f32;
                delay = (delay * dly.xfade as f32 / REVERB_XFADE as f32) + sample_xf;

                dly.output_pos_xf = (dly.output_pos_xf + 1) % dly.buffer_size;

                dly.xfade -= 1;
                if dly.xfade == 0 {
                    dly.output_pos = dly.output_pos_xf;
                }
            }

            // Feedback — no clipping, input was pre-scaled for headroom
            let val = vlr + dly.feedback * delay;

            // Lowpass
            let valt = if dly.lp_enabled {
                let valt = (dly.lp0 + val) * 0.5;
                dly.lp0 = val;
                valt
            } else {
                val
            };

            // Write to buffer
            voutm = valt;
            dly.buffer[dly.input_pos] = valt;
        } else {
            voutm = 0.0;
            dly.buffer[dly.input_pos] = 0.0;
            dly.lp0 = 0.0;
        }

        dly.move_pointer();
        voutm
    }

    // -------------------------------------------------------------------------
    //  DLY_DoDelay — Mono Echo with Feedback
    // -------------------------------------------------------------------------

    #[inline]
    fn do_delay(&mut self, left: &mut [f32], right: &mut [f32], delay_mix: f32) {
        let dly = &mut self.mono_dly;

        if dly.delay_samples == 0 {
            return;
        }

        let input_scale = 1.0 / (1.0 + dly.feedback);

        for i in 0..left.len() {
            let delay = dly.buffer[dly.output_pos];

            if delay.abs() > 1e-10 || left[i].abs() > 1e-10 || right[i].abs() > 1e-10 {
                // Mono downmix, scaled down for headroom
                let mut val = (left[i] + right[i]) * 0.5 * input_scale + dly.feedback * delay;

                // 3-tap lowpass
                if dly.lp_enabled {
                    val = (dly.lp0 + dly.lp1 + val) / 3.0;
                    dly.lp0 = dly.lp1;
                    dly.lp1 = val;
                }

                // Write to delay line
                dly.buffer[dly.input_pos] = val;

                // Additive wet mix
                let wet = val * delay_mix;
                left[i] += wet;
                right[i] += wet;
            } else {
                dly.buffer[dly.input_pos] = 0.0;
                dly.lp0 = 0.0;
                dly.lp1 = 0.0;
                dly.lp2 = 0.0;
            }

            dly.move_pointer();
        }
    }

    // -------------------------------------------------------------------------
    //  DLY_DoStereoDelay — Stereo Widening (Haas Effect)
    // -------------------------------------------------------------------------

    #[inline]
    fn do_stereo_delay(&mut self, left: &mut [f32], _right: &mut [f32]) {
        let dly = &mut self.stereo_dly;

        if dly.delay_samples == 0 {
            return;
        }

        for i in 0..left.len() {
            // Modulation
            if dly.modulation > 0 {
                dly.mod_cur -= 1;
                if dly.mod_cur < 0 {
                    dly.mod_cur = dly.modulation;
                }
            }

            // Read delayed left sample
            let mut delay = dly.buffer[dly.output_pos];

            if delay.abs() > 1e-10 || left[i].abs() > 1e-10 || dly.xfade != 0 {
                // Trigger crossfade wobble
                if dly.xfade == 0 && dly.mod_cur == 0 && dly.modulation > 0 {
                    let rand_val: u32 = self.rng.gen_range(0..=255);
                    let offset = (rand_val as usize * dly.delay_samples) / 512;
                    dly.output_pos_xf = (dly.output_pos + offset) % dly.buffer_size;
                    dly.xfade = STEREO_XFADE;
                }

                dly.output_pos_xf %= dly.buffer_size;

                // Crossfade
                if dly.xfade != 0 {
                    let sample_xf = dly.buffer[dly.output_pos_xf]
                        * (STEREO_XFADE - dly.xfade) as f32
                        / STEREO_XFADE as f32;
                    delay = sample_xf + (delay * dly.xfade as f32 / STEREO_XFADE as f32);

                    dly.output_pos_xf = (dly.output_pos_xf + 1) % dly.buffer_size;

                    dly.xfade -= 1;
                    if dly.xfade == 0 {
                        dly.output_pos = dly.output_pos_xf;
                    }
                }

                // Record current left, replace with delayed
                dly.buffer[dly.input_pos] = left[i].clamp(-1.0, 1.0);
                left[i] = delay;
            } else {
                dly.buffer[dly.input_pos] = 0.0;
            }

            dly.move_pointer();
        }
    }
}
