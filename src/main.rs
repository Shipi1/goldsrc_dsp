/// GoldSrc Reverb DSP — Offline CLI validator
///
/// Reads a WAV file, runs it through the DSP library, and writes the result.
///
/// Usage:
///     dsp_offline input.wav --room 5 --output output.wav
///     dsp_offline input.wav --room 14       # underwater
///     dsp_offline input.wav --all            # render all 29 presets
use std::path::{Path, PathBuf};

use clap::Parser;
use hound::{SampleFormat, WavReader, WavSpec, WavWriter};

use goldsrc_dsp::{preset_for_room, ClipMode, GoldSrcReverb, ROOM_NAMES};

// =============================================================================
//  CLI
// =============================================================================

#[derive(Parser, Debug)]
#[command(name = "dsp_offline")]
#[command(about = "GoldSrc Reverb DSP — Offline Tester")]
#[command(after_help = r#"
Examples:
  dsp_offline drums.wav --room 5
  dsp_offline vocals.wav --room 14 --output wet_vocals.wav
  dsp_offline snare.wav --all
"#)]
struct Cli {
    /// Input WAV file path
    input: PathBuf,

    /// Room type preset (0-28, default: 5)
    #[arg(long, default_value_t = 5)]
    room: usize,

    /// Reverb dry/wet mix (0.0=dry, 1.0=wet, default: 0.17 = GoldSrc)
    #[arg(long, default_value_t = 0.17)]
    mix: f32,

    /// Echo wet amount (0.0=off, 1.0=full, default: 0.25 = GoldSrc)
    #[arg(long, default_value_t = 0.25)]
    delay_mix: f32,

    /// Output clipping mode: hard or soft
    #[arg(long, default_value = "hard")]
    clip: String,

    /// Output WAV file path (default: input_roomN.wav)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Render all 29 room presets
    #[arg(long)]
    all: bool,
}

// =============================================================================
//  WAV I/O
// =============================================================================

/// Load a WAV file and return (left, right, sample_rate) as f32 arrays.
fn load_wav(path: &Path) -> (Vec<f32>, Vec<f32>, u32) {
    let reader = WavReader::open(path)
        .unwrap_or_else(|e| panic!("ERROR: Cannot open '{}': {}", path.display(), e));

    let spec = reader.spec();
    let sr = spec.sample_rate;
    let channels = spec.channels as usize;

    let samples: Vec<f32> = match spec.sample_format {
        SampleFormat::Int => {
            let max_val = (1i64 << (spec.bits_per_sample - 1)) as f32;
            reader
                .into_samples::<i32>()
                .map(|s| s.expect("Failed to read sample") as f32 / max_val)
                .collect()
        }
        SampleFormat::Float => reader
            .into_samples::<f32>()
            .map(|s| s.expect("Failed to read sample"))
            .collect(),
    };

    if channels == 1 {
        // Mono → duplicate to stereo
        (samples.clone(), samples, sr)
    } else {
        // Deinterleave
        let mut left = Vec::with_capacity(samples.len() / channels);
        let mut right = Vec::with_capacity(samples.len() / channels);
        for chunk in samples.chunks(channels) {
            left.push(chunk[0]);
            right.push(if channels > 1 { chunk[1] } else { chunk[0] });
        }
        (left, right, sr)
    }
}

/// Save stereo f32 arrays to a WAV file (PCM16).
fn save_wav(path: &Path, left: &[f32], right: &[f32], sample_rate: u32) {
    let spec = WavSpec {
        channels: 2,
        sample_rate,
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    };

    let mut writer = WavWriter::create(path, spec)
        .unwrap_or_else(|e| panic!("ERROR: Cannot create '{}': {}", path.display(), e));

    for i in 0..left.len() {
        let l = (left[i] * 32767.0).round().clamp(-32768.0, 32767.0) as i16;
        let r = (right[i] * 32767.0).round().clamp(-32768.0, 32767.0) as i16;
        writer.write_sample(l).unwrap();
        writer.write_sample(r).unwrap();
    }

    writer.finalize().unwrap();
    println!("  Written: {}", path.display());
}

// =============================================================================
//  Processing
// =============================================================================

fn process_room(
    input_path: &Path,
    output_path: &Path,
    room_type: usize,
    reverb_mix: f32,
    delay_mix: f32,
    clip_mode: ClipMode,
) {
    let (left, right, sr) = load_wav(input_path);

    let name = if room_type < ROOM_NAMES.len() {
        ROOM_NAMES[room_type]
    } else {
        "unknown"
    };

    println!(
        "  Processing room {} ({}) at {}Hz, reverb={:.0}%, delay={:.0}%, clip={:?}, {} samples...",
        room_type,
        name,
        sr,
        reverb_mix * 100.0,
        delay_mix * 100.0,
        clip_mode,
        left.len()
    );

    let mut reverb = GoldSrcReverb::new(sr);
    reverb.set_room_type(preset_for_room(room_type));
    reverb.set_reverb_mix(reverb_mix);
    reverb.set_delay_mix(delay_mix);
    reverb.set_clip_mode(clip_mode);

    let mut out_l = vec![0.0f32; left.len()];
    let mut out_r = vec![0.0f32; left.len()];

    reverb.process(&left, &right, &mut out_l, &mut out_r);

    save_wav(output_path, &out_l, &out_r, sr);
}

fn parse_clip_mode(s: &str) -> ClipMode {
    match s {
        "soft" => ClipMode::Soft,
        _ => ClipMode::Hard,
    }
}

// =============================================================================
//  Main
// =============================================================================

fn main() {
    let cli = Cli::parse();

    if !cli.input.is_file() {
        eprintln!("ERROR: File not found: {}", cli.input.display());
        std::process::exit(1);
    }

    let clip_mode = parse_clip_mode(&cli.clip);
    let stem = cli.input.file_stem().unwrap().to_string_lossy().to_string();
    let ext = cli
        .input
        .extension()
        .map(|e| e.to_string_lossy().to_string())
        .unwrap_or_else(|| "wav".to_string());

    if cli.all {
        println!("Processing all {} presets...", ROOM_NAMES.len());

        let out_dir = cli
            .input
            .parent()
            .unwrap_or(Path::new("."))
            .join(format!("{}_presets", stem));
        std::fs::create_dir_all(&out_dir).expect("Failed to create output directory");

        for room_id in 0..ROOM_NAMES.len() {
            let name = ROOM_NAMES[room_id];
            let out_path = out_dir.join(format!("{:02}_{}.{}", room_id, name, ext));
            process_room(
                &cli.input,
                &out_path,
                room_id,
                cli.mix,
                cli.delay_mix,
                clip_mode,
            );
        }

        println!("\nAll presets written to: {}", out_dir.display());
    } else {
        let out_path = if let Some(ref output) = cli.output {
            output.clone()
        } else {
            let name = if cli.room < ROOM_NAMES.len() {
                ROOM_NAMES[cli.room]
            } else {
                "unknown"
            };
            cli.input
                .parent()
                .unwrap_or(Path::new("."))
                .join(format!("{}_{}.{}", stem, name, ext))
        };

        process_room(
            &cli.input,
            &out_path,
            cli.room,
            cli.mix,
            cli.delay_mix,
            clip_mode,
        );
    }

    println!("Done.");
}
