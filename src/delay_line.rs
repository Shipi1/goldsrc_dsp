/// Circular delay buffer with crossfade modulation and lowpass filter.
///
/// Direct port of the Python `DelayLine` class from `goldsrc_reverb.py`.
/// All arithmetic is f32 to match the f32 processing path.

#[derive(Clone)]
#[allow(dead_code)]
pub struct DelayLine {
    pub sample_rate: u32,
    pub buffer: Vec<f32>,
    pub buffer_size: usize,

    // Circular buffer pointers
    pub input_pos: usize,
    pub output_pos: usize,

    // Crossfade state
    pub output_pos_xf: usize,
    pub xfade: i32,

    // Delay settings
    pub delay_samples: usize,
    pub feedback: f32,

    // Lowpass filter
    pub lp_enabled: bool,
    pub lp0: f32,
    pub lp1: f32,
    pub lp2: f32,

    // Modulation
    pub modulation: i32,
    pub mod_cur: i32,
}

impl DelayLine {
    /// Create a new delay line with the given maximum delay in seconds.
    pub fn new(max_delay_sec: f32, sample_rate: u32) -> Self {
        let buffer_size = (max_delay_sec * sample_rate as f32) as usize + 1;

        DelayLine {
            sample_rate,
            buffer: vec![0.0f32; buffer_size],
            buffer_size,
            input_pos: 0,
            output_pos: 0,
            output_pos_xf: 0,
            xfade: 0,
            delay_samples: 0,
            feedback: 0.0,
            lp_enabled: true,
            lp0: 0.0,
            lp1: 0.0,
            lp2: 0.0,
            modulation: 0,
            mod_cur: 0,
        }
    }

    /// Advance circular buffer pointers by one sample.
    #[inline]
    pub fn move_pointer(&mut self) {
        self.input_pos = (self.input_pos + 1) % self.buffer_size;
        self.output_pos = (self.output_pos + 1) % self.buffer_size;
    }

    /// Clear the buffer and filter state.
    pub fn reset(&mut self) {
        self.buffer.fill(0.0);
        self.lp0 = 0.0;
        self.lp1 = 0.0;
        self.lp2 = 0.0;
        self.xfade = 0;
    }
}
