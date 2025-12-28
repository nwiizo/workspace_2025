use noise::{NoiseFn, Perlin};
use rand::Rng;

/// Available pattern modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatternMode {
    /// Classic Perlin noise flowing gradients
    Perlin,
    /// Plasma effect with sine waves
    Plasma,
    /// Cellular/Voronoi-like patterns
    Cellular,
    /// Wave interference patterns
    Wave,
    /// Fractal Brownian Motion - layered noise
    Fbm,
    /// Domain warping - distorted noise
    Warp,
    /// Lissajous curves - oscilloscope patterns
    Lissajous,
    /// Kaleidoscope - mirrored symmetry
    Kaleidoscope,
    /// Spiral patterns
    Spiral,
    /// Checkerboard with noise
    Checker,
    /// Walking stick figures
    Walker,
    /// Metaballs - organic blob merging
    Metaballs,
    /// Julia set fractal
    Julia,
    /// Truchet tiles - maze-like patterns
    Truchet,
    /// Matrix rain effect
    Matrix,
    /// Fire effect
    Fire,
    /// Moiré interference patterns
    Moire,
    /// Random mode switches between patterns
    Random,
}

impl PatternMode {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "perlin" => Some(Self::Perlin),
            "plasma" => Some(Self::Plasma),
            "cellular" => Some(Self::Cellular),
            "wave" => Some(Self::Wave),
            "fbm" => Some(Self::Fbm),
            "warp" => Some(Self::Warp),
            "lissajous" => Some(Self::Lissajous),
            "kaleidoscope" | "kaleido" => Some(Self::Kaleidoscope),
            "spiral" => Some(Self::Spiral),
            "checker" => Some(Self::Checker),
            "walker" | "human" | "walk" => Some(Self::Walker),
            "metaballs" | "meta" | "blobs" => Some(Self::Metaballs),
            "julia" | "fractal" => Some(Self::Julia),
            "truchet" | "maze" | "tiles" => Some(Self::Truchet),
            "matrix" | "rain" => Some(Self::Matrix),
            "fire" | "flame" => Some(Self::Fire),
            "moire" | "interference" => Some(Self::Moire),
            "random" => Some(Self::Random),
            _ => None,
        }
    }

    pub fn all_modes() -> &'static [PatternMode] {
        &[
            PatternMode::Perlin,
            PatternMode::Plasma,
            PatternMode::Cellular,
            PatternMode::Wave,
            PatternMode::Fbm,
            PatternMode::Warp,
            PatternMode::Lissajous,
            PatternMode::Kaleidoscope,
            PatternMode::Spiral,
            PatternMode::Checker,
            PatternMode::Walker,
            PatternMode::Metaballs,
            PatternMode::Julia,
            PatternMode::Truchet,
            PatternMode::Matrix,
            PatternMode::Fire,
            PatternMode::Moire,
        ]
    }
}

impl std::fmt::Display for PatternMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Perlin => write!(f, "perlin"),
            Self::Plasma => write!(f, "plasma"),
            Self::Cellular => write!(f, "cellular"),
            Self::Wave => write!(f, "wave"),
            Self::Fbm => write!(f, "fbm"),
            Self::Warp => write!(f, "warp"),
            Self::Lissajous => write!(f, "lissajous"),
            Self::Kaleidoscope => write!(f, "kaleidoscope"),
            Self::Spiral => write!(f, "spiral"),
            Self::Checker => write!(f, "checker"),
            Self::Walker => write!(f, "walker"),
            Self::Metaballs => write!(f, "metaballs"),
            Self::Julia => write!(f, "julia"),
            Self::Truchet => write!(f, "truchet"),
            Self::Matrix => write!(f, "matrix"),
            Self::Fire => write!(f, "fire"),
            Self::Moire => write!(f, "moire"),
            Self::Random => write!(f, "random"),
        }
    }
}

/// Configuration for mosaic pattern generation
#[derive(Debug, Clone)]
pub struct MosaicConfig {
    /// Width of the video in pixels
    pub width: u32,
    /// Height of the video in pixels
    pub height: u32,
    /// Base size of each mosaic block in pixels
    pub block_size: u32,
    /// Interval in seconds between pattern changes
    pub change_interval: f64,
    /// Frames per second
    pub fps: u32,
    /// Pattern mode
    pub pattern_mode: PatternMode,
}

impl MosaicConfig {
    /// Create a new MosaicConfig with the given parameters
    pub fn new(width: u32, height: u32, block_size: u32, change_interval: f64, fps: u32) -> Self {
        Self {
            width,
            height,
            block_size,
            change_interval,
            fps,
            pattern_mode: PatternMode::Random,
        }
    }

    /// Set pattern mode
    pub fn with_pattern_mode(mut self, mode: PatternMode) -> Self {
        self.pattern_mode = mode;
        self
    }

    /// Calculate the number of blocks horizontally
    pub fn blocks_x(&self) -> u32 {
        (self.width + self.block_size - 1) / self.block_size
    }

    /// Calculate the number of blocks vertically
    pub fn blocks_y(&self) -> u32 {
        (self.height + self.block_size - 1) / self.block_size
    }

    /// Calculate frames per pattern change
    pub fn frames_per_change(&self) -> u32 {
        (self.change_interval * self.fps as f64).round() as u32
    }
}

/// Represents an RGB color
#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    /// Create a new color
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Create color from HSV values (h: 0-360, s: 0-1, v: 0-1)
    pub fn from_hsv(h: f64, s: f64, v: f64) -> Self {
        let h = h % 360.0;
        let h = if h < 0.0 { h + 360.0 } else { h };
        let c = v * s;
        let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
        let m = v - c;

        let (r, g, b) = if h < 60.0 {
            (c, x, 0.0)
        } else if h < 120.0 {
            (x, c, 0.0)
        } else if h < 180.0 {
            (0.0, c, x)
        } else if h < 240.0 {
            (0.0, x, c)
        } else if h < 300.0 {
            (x, 0.0, c)
        } else {
            (c, 0.0, x)
        };

        Self {
            r: ((r + m) * 255.0).clamp(0.0, 255.0) as u8,
            g: ((g + m) * 255.0).clamp(0.0, 255.0) as u8,
            b: ((b + m) * 255.0).clamp(0.0, 255.0) as u8,
        }
    }
}

/// A metaball (organic blob)
#[derive(Clone)]
struct Metaball {
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    radius: f64,
}

impl Metaball {
    fn new(rng: &mut rand::rngs::ThreadRng) -> Self {
        Self {
            x: rng.gen_range(0.2..0.8),
            y: rng.gen_range(0.2..0.8),
            vx: rng.gen_range(-0.01..0.01),
            vy: rng.gen_range(-0.01..0.01),
            radius: rng.gen_range(0.08..0.2),
        }
    }

    fn update(&mut self) {
        self.x += self.vx;
        self.y += self.vy;

        // Bounce off walls
        if self.x < 0.1 || self.x > 0.9 {
            self.vx = -self.vx;
        }
        if self.y < 0.1 || self.y > 0.9 {
            self.vy = -self.vy;
        }
    }

    fn influence(&self, px: f64, py: f64) -> f64 {
        let dx = px - self.x;
        let dy = py - self.y;
        let dist_sq = dx * dx + dy * dy;
        if dist_sq < 0.0001 {
            return 100.0;
        }
        self.radius * self.radius / dist_sq
    }
}

/// A column in the matrix rain effect
#[derive(Clone)]
struct MatrixColumn {
    x: f64,
    y: f64,
    speed: f64,
    length: usize,
    chars: Vec<u8>,
}

impl MatrixColumn {
    fn new(rng: &mut rand::rngs::ThreadRng, x_pos: f64) -> Self {
        let length = rng.gen_range(5..20);
        let chars: Vec<u8> = (0..length).map(|_| rng.gen_range(0..=255)).collect();
        Self {
            x: x_pos,
            y: rng.gen_range(-1.0..0.0),
            speed: rng.gen_range(0.01..0.04),
            length,
            chars,
        }
    }

    fn update(&mut self, rng: &mut rand::rngs::ThreadRng) {
        self.y += self.speed;
        if self.y > 1.5 {
            self.y = rng.gen_range(-0.5..0.0);
            self.speed = rng.gen_range(0.01..0.04);
        }
        // Randomly change some characters
        if rng.gen_range(0..10) < 2 {
            let idx = rng.gen_range(0..self.chars.len());
            self.chars[idx] = rng.gen_range(0..=255);
        }
    }
}

/// A walking stick figure
#[derive(Clone)]
struct Walker {
    /// X position (0.0 - 1.0)
    x: f64,
    /// Y position (0.0 - 1.0) - vertical position (ground level)
    y: f64,
    /// Walking speed
    speed: f64,
    /// Walking direction (1.0 = right, -1.0 = left)
    direction: f64,
    /// Scale factor
    scale: f64,
    /// Animation phase
    phase: f64,
    /// Hue for this walker
    hue: f64,
}

impl Walker {
    fn new(rng: &mut rand::rngs::ThreadRng) -> Self {
        let direction = if rng.gen_range(0..2) == 0 { 1.0 } else { -1.0 };
        Self {
            x: rng.gen_range(0.0..1.0),
            y: rng.gen_range(0.5..0.85), // Ground level range
            speed: rng.gen_range(0.002..0.008),
            direction,
            scale: rng.gen_range(0.08..0.2),
            phase: rng.gen_range(0.0..std::f64::consts::PI * 2.0),
            hue: rng.gen_range(0.0..360.0),
        }
    }

    /// Update walker position
    fn update(&mut self) {
        self.x += self.speed * self.direction;
        self.phase += 0.15;

        // Wrap around screen
        if self.x > 1.2 {
            self.x = -0.2;
        } else if self.x < -0.2 {
            self.x = 1.2;
        }
    }

    /// Check if a point is part of this walker's body
    /// Returns Some(brightness) if part of body, None otherwise
    fn is_body_part(&self, nx: f64, ny: f64, time: f64) -> Option<f64> {
        // Transform to walker-local coordinates
        let lx = (nx - self.x) / self.scale;
        let ly = (ny - self.y) / self.scale;

        // Walking animation
        let walk_cycle = (self.phase + time * 5.0).sin();
        let walk_cycle2 = (self.phase + time * 5.0 + std::f64::consts::PI).sin();

        // Head (circle at top)
        let head_y = -0.8;
        let head_radius = 0.15;
        let head_dist = ((lx * lx) + ((ly - head_y) * (ly - head_y))).sqrt();
        if head_dist < head_radius {
            return Some(1.0);
        }

        // Body (vertical line)
        let body_top = -0.65;
        let body_bottom = -0.2;
        let body_width = 0.06;
        if lx.abs() < body_width && ly > body_top && ly < body_bottom {
            return Some(0.9);
        }

        // Arms (animated)
        let arm_y = -0.5;
        let arm_swing = walk_cycle * 0.3;
        // Left arm
        let arm_l_end_x = -0.35;
        let arm_l_end_y = arm_y + arm_swing * 0.2;
        if Self::point_near_line(lx, ly, 0.0, arm_y, arm_l_end_x, arm_l_end_y, 0.05) {
            return Some(0.85);
        }
        // Right arm
        let arm_r_end_x = 0.35;
        let arm_r_end_y = arm_y - arm_swing * 0.2;
        if Self::point_near_line(lx, ly, 0.0, arm_y, arm_r_end_x, arm_r_end_y, 0.05) {
            return Some(0.85);
        }

        // Legs (animated walking)
        let hip_y = -0.2;
        let leg_length = 0.5;
        // Left leg
        let leg_l_swing = walk_cycle * 0.4;
        let leg_l_end_x = leg_l_swing * 0.3;
        let leg_l_end_y = hip_y + leg_length;
        if Self::point_near_line(lx, ly, 0.0, hip_y, leg_l_end_x, leg_l_end_y, 0.05) {
            return Some(0.8);
        }
        // Right leg
        let leg_r_swing = walk_cycle2 * 0.4;
        let leg_r_end_x = leg_r_swing * 0.3;
        let leg_r_end_y = hip_y + leg_length;
        if Self::point_near_line(lx, ly, 0.0, hip_y, leg_r_end_x, leg_r_end_y, 0.05) {
            return Some(0.8);
        }

        None
    }

    /// Check if point is near a line segment
    fn point_near_line(px: f64, py: f64, x1: f64, y1: f64, x2: f64, y2: f64, threshold: f64) -> bool {
        let dx = x2 - x1;
        let dy = y2 - y1;
        let len_sq = dx * dx + dy * dy;

        if len_sq < 0.0001 {
            return ((px - x1).powi(2) + (py - y1).powi(2)).sqrt() < threshold;
        }

        let t = ((px - x1) * dx + (py - y1) * dy) / len_sq;
        let t = t.clamp(0.0, 1.0);

        let closest_x = x1 + t * dx;
        let closest_y = y1 + t * dy;

        let dist = ((px - closest_x).powi(2) + (py - closest_y).powi(2)).sqrt();
        dist < threshold
    }
}

/// Mosaic pattern generator with multiple modes
pub struct MosaicGenerator {
    config: MosaicConfig,
    perlin: Perlin,
    rng: rand::rngs::ThreadRng,
    time: f64,
    base_hue: f64,
    hue_offset: f64,
    noise_scale: f64,
    frames_since_change: u32,
    current_mode: PatternMode,
    // For cellular pattern
    cell_centers: Vec<(f64, f64)>,
    // For dynamic block size
    block_size_variation: f64,
    // Wave parameters
    wave_freq_x: f64,
    wave_freq_y: f64,
    wave_phase: f64,
    // Lissajous parameters
    lissa_a: f64,
    lissa_b: f64,
    lissa_delta: f64,
    // Kaleidoscope segments
    kaleido_segments: u32,
    // Spiral parameters
    spiral_arms: u32,
    spiral_tightness: f64,
    // Walker parameters - multiple walkers
    walkers: Vec<Walker>,
    // Metaballs parameters
    metaballs: Vec<Metaball>,
    // Julia set parameters
    julia_c: (f64, f64),
    // Matrix rain columns
    matrix_columns: Vec<MatrixColumn>,
    // Fire buffer (heat values)
    fire_buffer: Vec<f64>,
    fire_width: usize,
    fire_height: usize,
    // Moire parameters
    moire_offset: f64,
}

impl MosaicGenerator {
    /// Create a new mosaic generator with the given configuration
    pub fn new(config: MosaicConfig) -> Self {
        let mut rng = rand::thread_rng();
        let seed: u32 = rng.gen_range(0..u32::MAX);
        let perlin = Perlin::new(seed);
        let hue_offset = rng.gen_range(0.0..360.0);

        let current_mode = if config.pattern_mode == PatternMode::Random {
            *PatternMode::all_modes()
                .get(rng.gen_range(0..PatternMode::all_modes().len()))
                .unwrap_or(&PatternMode::Perlin)
        } else {
            config.pattern_mode
        };

        // Generate random cell centers for cellular pattern
        let num_cells = rng.gen_range(8..20);
        let cell_centers: Vec<(f64, f64)> = (0..num_cells)
            .map(|_| (rng.gen_range(0.0..1.0), rng.gen_range(0.0..1.0)))
            .collect();

        let wave_freq_x = rng.gen_range(2.0..6.0);
        let wave_freq_y = rng.gen_range(2.0..6.0);

        // Lissajous parameters (frequency ratios that produce nice patterns)
        let lissa_ratios = [(1.0, 2.0), (2.0, 3.0), (3.0, 4.0), (3.0, 5.0), (4.0, 5.0)];
        let (lissa_a, lissa_b) = lissa_ratios[rng.gen_range(0..lissa_ratios.len())];
        let lissa_delta = rng.gen_range(0.0..std::f64::consts::PI);

        // Extract remaining values before moving rng
        let kaleido_segments = rng.gen_range(4..12);
        let spiral_arms = rng.gen_range(2..8);
        let spiral_tightness = rng.gen_range(3.0..8.0);

        // Create walking figures (3-7 walkers)
        let num_walkers = rng.gen_range(3..8);
        let walkers: Vec<Walker> = (0..num_walkers)
            .map(|_| Walker::new(&mut rng))
            .collect();

        // Create metaballs (4-8 blobs)
        let num_metaballs = rng.gen_range(4..9);
        let metaballs: Vec<Metaball> = (0..num_metaballs)
            .map(|_| Metaball::new(&mut rng))
            .collect();

        // Julia set constant - interesting values near Mandelbrot boundary
        let julia_constants = [
            (-0.7, 0.27015),
            (-0.4, 0.6),
            (0.285, 0.01),
            (-0.8, 0.156),
            (-0.70176, -0.3842),
            (0.355, 0.355),
        ];
        let julia_c = julia_constants[rng.gen_range(0..julia_constants.len())];

        // Matrix rain columns
        let num_columns = 40;
        let matrix_columns: Vec<MatrixColumn> = (0..num_columns)
            .map(|i| MatrixColumn::new(&mut rng, i as f64 / num_columns as f64))
            .collect();

        // Fire buffer - initialize with some heat at the bottom
        let fire_width = (config.width / config.block_size) as usize;
        let fire_height = (config.height / config.block_size) as usize;
        let mut fire_buffer = vec![0.0; fire_width * fire_height];
        // Initialize bottom row with fire
        for x in 0..fire_width {
            let idx = (fire_height - 1) * fire_width + x;
            if idx < fire_buffer.len() {
                fire_buffer[idx] = rng.gen_range(0.5..1.0);
            }
        }

        // Moire offset
        let moire_offset = rng.gen_range(0.0..std::f64::consts::PI * 2.0);

        Self {
            config,
            perlin,
            rng,
            time: 0.0,
            base_hue: 0.0,
            hue_offset,
            noise_scale: 0.08,
            frames_since_change: 0,
            current_mode,
            cell_centers,
            block_size_variation: 0.0,
            wave_freq_x,
            wave_freq_y,
            wave_phase: 0.0,
            lissa_a,
            lissa_b,
            lissa_delta,
            kaleido_segments,
            spiral_arms,
            spiral_tightness,
            walkers,
            metaballs,
            julia_c,
            matrix_columns,
            fire_buffer,
            fire_width,
            fire_height,
            moire_offset,
        }
    }

    /// Update parameters for pattern evolution
    fn update_pattern(&mut self) {
        // Slowly shift base hue for color evolution
        self.base_hue = (self.base_hue + 3.0) % 360.0;

        // Update block size variation
        self.block_size_variation = self.rng.gen_range(-0.3..0.3);

        // Occasionally change parameters for variety
        if self.rng.gen_range(0..100) < 15 {
            let seed: u32 = self.rng.gen_range(0..u32::MAX);
            self.perlin = Perlin::new(seed);
            self.hue_offset = self.rng.gen_range(0.0..360.0);
            self.noise_scale = self.rng.gen_range(0.04..0.12);

            // Change wave parameters
            self.wave_freq_x = self.rng.gen_range(2.0..8.0);
            self.wave_freq_y = self.rng.gen_range(2.0..8.0);

            // Change lissajous parameters
            let lissa_ratios = [(1.0, 2.0), (2.0, 3.0), (3.0, 4.0), (3.0, 5.0), (4.0, 5.0), (5.0, 6.0)];
            let (a, b) = lissa_ratios[self.rng.gen_range(0..lissa_ratios.len())];
            self.lissa_a = a;
            self.lissa_b = b;
            self.lissa_delta = self.rng.gen_range(0.0..std::f64::consts::PI);

            // Change kaleidoscope segments
            self.kaleido_segments = self.rng.gen_range(4..12);

            // Change spiral parameters
            self.spiral_arms = self.rng.gen_range(2..8);
            self.spiral_tightness = self.rng.gen_range(3.0..8.0);
        }

        // For Random mode, occasionally switch pattern
        if self.config.pattern_mode == PatternMode::Random && self.rng.gen_range(0..100) < 5 {
            self.current_mode = *PatternMode::all_modes()
                .get(self.rng.gen_range(0..PatternMode::all_modes().len()))
                .unwrap_or(&PatternMode::Perlin);

            // Regenerate cell centers when switching to cellular
            if self.current_mode == PatternMode::Cellular {
                let num_cells = self.rng.gen_range(8..20);
                self.cell_centers = (0..num_cells)
                    .map(|_| (self.rng.gen_range(0.0..1.0), self.rng.gen_range(0.0..1.0)))
                    .collect();
            }
        }

        // Update walkers position
        for walker in &mut self.walkers {
            walker.update();
        }

        // Update metaballs
        for metaball in &mut self.metaballs {
            metaball.update();
        }

        // Update matrix columns
        for column in &mut self.matrix_columns {
            column.update(&mut self.rng);
        }

        // Update moire offset
        self.moire_offset += 0.02;

        // Update fire (propagate heat upward)
        self.update_fire();
    }

    /// Update fire simulation
    fn update_fire(&mut self) {
        let w = self.fire_width;
        let h = self.fire_height;

        // Set bottom row on fire with random intensity
        for x in 0..w {
            let idx = (h - 1) * w + x;
            self.fire_buffer[idx] = self.rng.gen_range(0.7..1.0);
        }

        // Propagate fire upward with cooling
        for y in 0..(h - 1) {
            for x in 0..w {
                let idx = y * w + x;
                let below = (y + 1) * w + x;
                let left = if x > 0 { (y + 1) * w + x - 1 } else { below };
                let right = if x < w - 1 { (y + 1) * w + x + 1 } else { below };

                let avg = (self.fire_buffer[below]
                    + self.fire_buffer[left]
                    + self.fire_buffer[right]) / 3.0;

                // Cooling factor increases with height
                let cooling = 0.02 + (h - y) as f64 * 0.005;
                self.fire_buffer[idx] = (avg - cooling).max(0.0);
            }
        }
    }

    /// Write the next frame directly to a buffer (RGB24 format)
    pub fn write_frame_to_buffer(&mut self, buffer: &mut [u8]) {
        // Check if we need to update pattern
        if self.frames_since_change >= self.config.frames_per_change() {
            self.update_pattern();
            self.frames_since_change = 0;
        }
        self.frames_since_change += 1;

        // Advance time for animation
        self.time += 0.025;
        self.wave_phase += 0.03;

        // Render the frame to the buffer
        self.render_to_buffer(buffer);
    }

    /// Get effective block size with variation
    fn get_effective_block_size(&self, block_x: usize, block_y: usize) -> usize {
        let base = self.config.block_size as f64;
        let noise_val = self.perlin.get([
            block_x as f64 * 0.3,
            block_y as f64 * 0.3,
            self.time * 0.5,
        ]);
        let variation = 1.0 + self.block_size_variation * noise_val;
        (base * variation).clamp(32.0, 128.0) as usize
    }

    /// Get color based on current pattern mode
    fn get_block_color(&self, block_x: usize, block_y: usize, width: usize, height: usize) -> Color {
        match self.current_mode {
            PatternMode::Perlin => self.get_perlin_color(block_x, block_y),
            PatternMode::Plasma => self.get_plasma_color(block_x, block_y, width, height),
            PatternMode::Cellular => self.get_cellular_color(block_x, block_y, width, height),
            PatternMode::Wave => self.get_wave_color(block_x, block_y, width, height),
            PatternMode::Fbm => self.get_fbm_color(block_x, block_y),
            PatternMode::Warp => self.get_warp_color(block_x, block_y),
            PatternMode::Lissajous => self.get_lissajous_color(block_x, block_y, width, height),
            PatternMode::Kaleidoscope => self.get_kaleidoscope_color(block_x, block_y, width, height),
            PatternMode::Spiral => self.get_spiral_color(block_x, block_y, width, height),
            PatternMode::Checker => self.get_checker_color(block_x, block_y),
            PatternMode::Walker => self.get_walker_color(block_x, block_y, width, height),
            PatternMode::Metaballs => self.get_metaballs_color(block_x, block_y, width, height),
            PatternMode::Julia => self.get_julia_color(block_x, block_y, width, height),
            PatternMode::Truchet => self.get_truchet_color(block_x, block_y, width, height),
            PatternMode::Matrix => self.get_matrix_color(block_x, block_y, width, height),
            PatternMode::Fire => self.get_fire_color(block_x, block_y),
            PatternMode::Moire => self.get_moire_color(block_x, block_y, width, height),
            PatternMode::Random => self.get_perlin_color(block_x, block_y), // Fallback
        }
    }

    /// Perlin noise based color
    fn get_perlin_color(&self, block_x: usize, block_y: usize) -> Color {
        let x = block_x as f64 * self.noise_scale;
        let y = block_y as f64 * self.noise_scale;

        // Multiple octaves of noise
        let noise1 = self.perlin.get([x, y, self.time]);
        let noise2 = self.perlin.get([x * 2.0, y * 2.0, self.time * 0.5]);
        let noise3 = self.perlin.get([x * 0.5 + 100.0, y * 0.5 + 100.0, self.time * 0.3]);

        let combined = (noise1 * 0.5 + noise2 * 0.3 + noise3 * 0.2 + 1.0) / 2.0;
        let hue = (self.base_hue + self.hue_offset + combined * 120.0) % 360.0;

        let sat_noise = self.perlin.get([x + 50.0, y + 50.0, self.time * 0.7]);
        let val_noise = self.perlin.get([x + 100.0, y + 100.0, self.time * 0.4]);

        let saturation = 0.6 + (sat_noise + 1.0) * 0.2;
        let value = 0.5 + (val_noise + 1.0) * 0.25;

        Color::from_hsv(hue, saturation.clamp(0.0, 1.0), value.clamp(0.0, 1.0))
    }

    /// Plasma effect using sine waves
    fn get_plasma_color(&self, block_x: usize, block_y: usize, width: usize, height: usize) -> Color {
        let nx = block_x as f64 / width as f64;
        let ny = block_y as f64 / height as f64;

        // Classic plasma formula with multiple sine waves
        let v1 = (nx * 10.0 + self.time).sin();
        let v2 = ((ny * 10.0 + self.time * 0.7).sin() + (nx * 5.0).cos()) / 2.0;
        let v3 = ((nx + ny + self.time * 0.5) * 5.0).sin();
        let cx = nx + 0.5 * (self.time * 0.3).sin();
        let cy = ny + 0.5 * (self.time * 0.4).cos();
        let v4 = ((cx * cx + cy * cy).sqrt() * 10.0 - self.time).sin();

        let combined = (v1 + v2 + v3 + v4) / 4.0;
        let normalized = (combined + 1.0) / 2.0;

        let hue = (self.base_hue + normalized * 180.0) % 360.0;
        let saturation = 0.7 + normalized * 0.3;
        let value = 0.6 + normalized * 0.4;

        Color::from_hsv(hue, saturation, value)
    }

    /// Cellular/Voronoi-like pattern
    fn get_cellular_color(&self, block_x: usize, block_y: usize, width: usize, height: usize) -> Color {
        let nx = block_x as f64 / width as f64;
        let ny = block_y as f64 / height as f64;

        let mut min_dist = f64::MAX;
        let mut second_dist = f64::MAX;
        let mut nearest_idx = 0;

        for (i, &(cx, cy)) in self.cell_centers.iter().enumerate() {
            let animated_cx = cx + 0.1 * (self.time * 0.5 + i as f64).sin();
            let animated_cy = cy + 0.1 * (self.time * 0.3 + i as f64 * 1.5).cos();

            let dx = nx - animated_cx;
            let dy = ny - animated_cy;
            let dist = (dx * dx + dy * dy).sqrt();

            if dist < min_dist {
                second_dist = min_dist;
                min_dist = dist;
                nearest_idx = i;
            } else if dist < second_dist {
                second_dist = dist;
            }
        }

        let edge = (second_dist - min_dist).clamp(0.0, 0.3) / 0.3;

        let hue = (self.base_hue + self.hue_offset + nearest_idx as f64 * 30.0) % 360.0;
        let saturation = 0.5 + edge * 0.4;
        let value = 0.4 + edge * 0.5;

        Color::from_hsv(hue, saturation, value)
    }

    /// Wave interference pattern
    fn get_wave_color(&self, block_x: usize, block_y: usize, width: usize, height: usize) -> Color {
        let nx = block_x as f64 / width as f64;
        let ny = block_y as f64 / height as f64;

        let wave1 = (nx * self.wave_freq_x * std::f64::consts::PI * 2.0 + self.wave_phase).sin();
        let wave2 = (ny * self.wave_freq_y * std::f64::consts::PI * 2.0 + self.wave_phase * 0.7).sin();

        let cx = nx - 0.5;
        let cy = ny - 0.5;
        let dist = (cx * cx + cy * cy).sqrt();
        let wave3 = (dist * 15.0 - self.wave_phase * 2.0).sin();

        let wave4 = ((nx + ny) * 8.0 + self.wave_phase * 0.5).sin();

        let combined = (wave1 + wave2 + wave3 * 0.5 + wave4 * 0.3) / 2.8;
        let normalized = (combined + 1.0) / 2.0;

        let noise = self.perlin.get([nx * 5.0, ny * 5.0, self.time * 0.3]);
        let final_val = (normalized + noise * 0.2).clamp(0.0, 1.0);

        let hue = (self.base_hue + final_val * 90.0) % 360.0;
        let saturation = 0.6 + final_val * 0.3;
        let value = 0.5 + final_val * 0.4;

        Color::from_hsv(hue, saturation, value)
    }

    /// Fractal Brownian Motion - layered noise with octaves
    fn get_fbm_color(&self, block_x: usize, block_y: usize) -> Color {
        let x = block_x as f64 * 0.05;
        let y = block_y as f64 * 0.05;

        // FBM with 5 octaves
        let mut amplitude = 1.0;
        let mut frequency = 1.0;
        let mut value = 0.0;
        let mut max_value = 0.0;
        let lacunarity = 2.0;
        let gain = 0.5;

        for _ in 0..5 {
            value += amplitude * self.perlin.get([
                x * frequency,
                y * frequency,
                self.time * 0.3 * frequency,
            ]);
            max_value += amplitude;
            amplitude *= gain;
            frequency *= lacunarity;
        }

        let normalized = (value / max_value + 1.0) / 2.0;

        // Turbulence effect using absolute value
        let turbulence = (value / max_value).abs();

        let hue = (self.base_hue + self.hue_offset + normalized * 120.0) % 360.0;
        let saturation = 0.5 + turbulence * 0.4;
        let value = 0.4 + normalized * 0.5;

        Color::from_hsv(hue, saturation.clamp(0.0, 1.0), value.clamp(0.0, 1.0))
    }

    /// Domain warping - distort coordinates with noise
    fn get_warp_color(&self, block_x: usize, block_y: usize) -> Color {
        let x = block_x as f64 * 0.06;
        let y = block_y as f64 * 0.06;

        // First layer of warping
        let warp_x1 = self.perlin.get([x, y, self.time * 0.2]);
        let warp_y1 = self.perlin.get([x + 5.2, y + 1.3, self.time * 0.2]);

        // Second layer of warping (warp the warp)
        let warp_x2 = self.perlin.get([
            x + 4.0 * warp_x1,
            y + 4.0 * warp_y1,
            self.time * 0.15,
        ]);
        let warp_y2 = self.perlin.get([
            x + 4.0 * warp_x1 + 1.7,
            y + 4.0 * warp_y1 + 9.2,
            self.time * 0.15,
        ]);

        // Final noise with double-warped coordinates
        let final_noise = self.perlin.get([
            x + 4.0 * warp_x2,
            y + 4.0 * warp_y2,
            self.time * 0.1,
        ]);

        let normalized = (final_noise + 1.0) / 2.0;

        let hue = (self.base_hue + normalized * 180.0) % 360.0;
        let saturation = 0.6 + normalized * 0.3;
        let value = 0.5 + normalized * 0.4;

        Color::from_hsv(hue, saturation, value)
    }

    /// Lissajous curve pattern
    fn get_lissajous_color(&self, block_x: usize, block_y: usize, width: usize, height: usize) -> Color {
        let nx = (block_x as f64 / width as f64) * 2.0 - 1.0;
        let ny = (block_y as f64 / height as f64) * 2.0 - 1.0;

        // Calculate distance to Lissajous curve
        let mut min_dist = f64::MAX;
        let steps = 200;

        for i in 0..steps {
            let t = (i as f64 / steps as f64) * std::f64::consts::PI * 2.0;
            let lx = (self.lissa_a * t + self.time * 0.5).sin();
            let ly = (self.lissa_b * t + self.lissa_delta + self.time * 0.3).sin();

            let dist = ((nx - lx).powi(2) + (ny - ly).powi(2)).sqrt();
            if dist < min_dist {
                min_dist = dist;
            }
        }

        // Create glow effect based on distance
        let glow = (-min_dist * 5.0).exp();
        let noise = self.perlin.get([nx * 3.0, ny * 3.0, self.time * 0.2]);

        let hue = (self.base_hue + self.hue_offset + glow * 60.0 + noise * 30.0) % 360.0;
        let saturation = 0.7 + glow * 0.3;
        let value = 0.2 + glow * 0.7;

        Color::from_hsv(hue, saturation.clamp(0.0, 1.0), value.clamp(0.0, 1.0))
    }

    /// Kaleidoscope pattern with mirror symmetry
    fn get_kaleidoscope_color(&self, block_x: usize, block_y: usize, width: usize, height: usize) -> Color {
        let cx = block_x as f64 / width as f64 - 0.5;
        let cy = block_y as f64 / height as f64 - 0.5;

        // Convert to polar coordinates
        let r = (cx * cx + cy * cy).sqrt();
        let mut theta = cy.atan2(cx);

        // Create kaleidoscope segments by folding the angle
        let segment_angle = std::f64::consts::PI * 2.0 / self.kaleido_segments as f64;
        theta = theta % segment_angle;
        if (theta / segment_angle * 2.0) as i32 % 2 == 1 {
            theta = segment_angle - theta;
        }

        // Convert back to cartesian for noise sampling
        let kx = r * theta.cos() + self.time * 0.1;
        let ky = r * theta.sin() + self.time * 0.1;

        // Sample noise at transformed coordinates
        let noise1 = self.perlin.get([kx * 5.0, ky * 5.0, self.time * 0.2]);
        let noise2 = self.perlin.get([kx * 10.0, ky * 10.0, self.time * 0.3]);

        let combined = (noise1 + noise2 * 0.5) / 1.5;
        let normalized = (combined + 1.0) / 2.0;

        let hue = (self.base_hue + self.hue_offset + r * 200.0 + normalized * 60.0) % 360.0;
        let saturation = 0.6 + normalized * 0.3;
        let value = 0.5 + normalized * 0.4;

        Color::from_hsv(hue, saturation, value)
    }

    /// Spiral pattern
    fn get_spiral_color(&self, block_x: usize, block_y: usize, width: usize, height: usize) -> Color {
        let cx = block_x as f64 / width as f64 - 0.5;
        let cy = block_y as f64 / height as f64 - 0.5;

        // Polar coordinates
        let r = (cx * cx + cy * cy).sqrt() * 2.0;
        let theta = cy.atan2(cx);

        // Spiral formula
        let spiral = (theta * self.spiral_arms as f64 + r * self.spiral_tightness - self.time * 2.0).sin();

        // Add noise for texture
        let noise = self.perlin.get([cx * 8.0, cy * 8.0, self.time * 0.3]);

        let combined = (spiral + noise * 0.3 + 1.0) / 2.0;

        let hue = (self.base_hue + self.hue_offset + r * 100.0) % 360.0;
        let saturation = 0.6 + combined * 0.3;
        let value = 0.4 + combined * 0.5;

        Color::from_hsv(hue, saturation, value)
    }

    /// Checker pattern with noise variation
    fn get_checker_color(&self, block_x: usize, block_y: usize) -> Color {
        let scale = 3.0;
        let x = block_x as f64 * self.noise_scale * scale;
        let y = block_y as f64 * self.noise_scale * scale;

        // Basic checker
        let checker = ((x + self.time * 0.5).floor() as i32 + (y + self.time * 0.3).floor() as i32) % 2;

        // Add noise variation
        let noise = self.perlin.get([x * 0.5, y * 0.5, self.time * 0.2]);

        let base_val = if checker == 0 { 0.3 } else { 0.7 };
        let final_val = (base_val + noise * 0.2).clamp(0.0, 1.0);

        let hue = (self.base_hue + self.hue_offset + noise * 60.0) % 360.0;
        let saturation = 0.5 + final_val * 0.3;
        let value = final_val;

        Color::from_hsv(hue, saturation, value)
    }

    /// Walker pattern - stick figures walking across the screen
    fn get_walker_color(&self, block_x: usize, block_y: usize, width: usize, height: usize) -> Color {
        let nx = block_x as f64 / width as f64;
        let ny = block_y as f64 / height as f64;

        // Check if this block is part of any walker
        for walker in &self.walkers {
            if let Some(brightness) = walker.is_body_part(nx, ny, self.time) {
                // Walker body color
                let hue = walker.hue;
                let saturation = 0.8;
                let value = brightness;
                return Color::from_hsv(hue, saturation, value);
            }
        }

        // Background - subtle animated gradient
        let bg_noise = self.perlin.get([nx * 3.0, ny * 3.0, self.time * 0.1]);
        let bg_normalized = (bg_noise + 1.0) / 2.0;

        // Dark background with subtle variation
        let hue = (self.base_hue + self.hue_offset + bg_normalized * 30.0) % 360.0;
        let saturation = 0.3 + bg_normalized * 0.2;
        let value = 0.1 + bg_normalized * 0.15; // Dark background

        Color::from_hsv(hue, saturation, value)
    }

    /// Metaballs - organic blob merging effect
    fn get_metaballs_color(&self, block_x: usize, block_y: usize, width: usize, height: usize) -> Color {
        let nx = block_x as f64 / width as f64;
        let ny = block_y as f64 / height as f64;

        // Sum influence from all metaballs
        let mut total_influence = 0.0;
        for metaball in &self.metaballs {
            total_influence += metaball.influence(nx, ny);
        }

        // Threshold creates the blob effect
        let threshold = 1.0;
        let normalized = (total_influence / threshold).min(1.0);

        // Color based on influence strength
        if normalized > 0.9 {
            // Inside blob - bright color
            let hue = (self.base_hue + self.hue_offset) % 360.0;
            Color::from_hsv(hue, 0.8, 0.9)
        } else if normalized > 0.5 {
            // Edge of blob - gradient
            let edge = (normalized - 0.5) / 0.4;
            let hue = (self.base_hue + self.hue_offset + edge * 30.0) % 360.0;
            Color::from_hsv(hue, 0.7 + edge * 0.2, 0.5 + edge * 0.4)
        } else {
            // Background with subtle noise
            let noise = self.perlin.get([nx * 5.0, ny * 5.0, self.time * 0.1]);
            let bg = 0.1 + (noise + 1.0) * 0.05;
            let hue = (self.base_hue + self.hue_offset + 180.0) % 360.0;
            Color::from_hsv(hue, 0.3, bg)
        }
    }

    /// Julia set fractal
    fn get_julia_color(&self, block_x: usize, block_y: usize, width: usize, height: usize) -> Color {
        // Map to complex plane with zoom animation
        let zoom = 1.5 + (self.time * 0.2).sin() * 0.3;
        let offset_x = (self.time * 0.1).cos() * 0.2;
        let offset_y = (self.time * 0.1).sin() * 0.2;

        let mut zx = (block_x as f64 / width as f64 - 0.5) * 3.0 / zoom + offset_x;
        let mut zy = (block_y as f64 / height as f64 - 0.5) * 3.0 / zoom + offset_y;

        let (cx, cy) = self.julia_c;

        let max_iter = 50;
        let mut iter = 0;

        while zx * zx + zy * zy < 4.0 && iter < max_iter {
            let tmp = zx * zx - zy * zy + cx;
            zy = 2.0 * zx * zy + cy;
            zx = tmp;
            iter += 1;
        }

        if iter == max_iter {
            // Inside the set - dark
            Color::from_hsv(0.0, 0.0, 0.05)
        } else {
            // Outside - color based on escape time
            let smooth = iter as f64 + 1.0 - (zx * zx + zy * zy).ln().ln() / 2.0_f64.ln();
            let hue = (self.base_hue + self.hue_offset + smooth * 10.0) % 360.0;
            let value = 0.5 + (smooth / max_iter as f64) * 0.5;
            Color::from_hsv(hue, 0.8, value.clamp(0.0, 1.0))
        }
    }

    /// Truchet tiles - maze-like patterns
    fn get_truchet_color(&self, block_x: usize, block_y: usize, width: usize, height: usize) -> Color {
        let nx = block_x as f64 / width as f64;
        let ny = block_y as f64 / height as f64;

        // Determine tile type based on noise
        let tile_x = (nx * 10.0).floor() as i32;
        let tile_y = (ny * 10.0).floor() as i32;
        let noise_val = self.perlin.get([tile_x as f64 * 0.5, tile_y as f64 * 0.5, self.time * 0.1]);
        let tile_type = noise_val > 0.0;

        // Position within tile (0-1)
        let tx = (nx * 10.0).fract();
        let ty = (ny * 10.0).fract();

        // Calculate distance to arc (quarter circle)
        let (cx, cy) = if tile_type { (0.0, 0.0) } else { (1.0, 1.0) };
        let dist1 = ((tx - cx).powi(2) + (ty - cy).powi(2)).sqrt();

        let (cx2, cy2) = if tile_type { (1.0, 1.0) } else { (0.0, 0.0) };
        let dist2 = ((tx - cx2).powi(2) + (ty - cy2).powi(2)).sqrt();

        // Arc radius is 0.5
        let arc_dist = (dist1 - 0.5).abs().min((dist2 - 0.5).abs());

        // Create line effect
        let line_width = 0.08;
        let on_line = arc_dist < line_width;

        if on_line {
            let hue = (self.base_hue + self.hue_offset) % 360.0;
            Color::from_hsv(hue, 0.7, 0.9)
        } else {
            let hue = (self.base_hue + self.hue_offset + 180.0) % 360.0;
            Color::from_hsv(hue, 0.3, 0.15)
        }
    }

    /// Matrix rain effect
    fn get_matrix_color(&self, block_x: usize, block_y: usize, width: usize, height: usize) -> Color {
        let nx = block_x as f64 / width as f64;
        let ny = block_y as f64 / height as f64;

        // Check if this position is part of any matrix column
        for column in &self.matrix_columns {
            let dx = (nx - column.x).abs();
            if dx < 0.03 {
                // Within column width
                let relative_y = ny - column.y;
                if relative_y >= 0.0 && relative_y < column.length as f64 * 0.05 {
                    let char_idx = (relative_y / 0.05) as usize;
                    if char_idx < column.chars.len() {
                        // Brightness fades toward bottom
                        let fade = 1.0 - (char_idx as f64 / column.length as f64);

                        // Head of column is brightest (white-ish)
                        if char_idx == 0 {
                            return Color::from_hsv(120.0, 0.3, 1.0);
                        }

                        // Green with varying brightness
                        return Color::from_hsv(120.0, 0.9, fade * 0.8);
                    }
                }
            }
        }

        // Dark background with slight noise
        let noise = self.perlin.get([nx * 20.0, ny * 20.0, self.time * 0.05]);
        let bg = 0.02 + (noise + 1.0) * 0.01;
        Color::from_hsv(120.0, 0.5, bg)
    }

    /// Fire effect
    fn get_fire_color(&self, block_x: usize, block_y: usize) -> Color {
        let idx = block_y * self.fire_width + block_x;
        let heat = if idx < self.fire_buffer.len() {
            self.fire_buffer[idx]
        } else {
            0.0
        };

        // Fire color palette: black -> red -> orange -> yellow -> white
        if heat < 0.2 {
            let v = heat / 0.2;
            Color::from_hsv(0.0, 1.0, v * 0.3)
        } else if heat < 0.4 {
            let t = (heat - 0.2) / 0.2;
            // Red to orange
            Color::from_hsv(t * 30.0, 1.0, 0.3 + t * 0.4)
        } else if heat < 0.7 {
            let t = (heat - 0.4) / 0.3;
            // Orange to yellow
            Color::from_hsv(30.0 + t * 30.0, 1.0 - t * 0.2, 0.7 + t * 0.2)
        } else {
            let t = (heat - 0.7) / 0.3;
            // Yellow to white
            Color::from_hsv(60.0, 1.0 - t * 0.8, 0.9 + t * 0.1)
        }
    }

    /// Moiré interference pattern
    fn get_moire_color(&self, block_x: usize, block_y: usize, width: usize, height: usize) -> Color {
        let nx = block_x as f64 / width as f64;
        let ny = block_y as f64 / height as f64;

        // Center coordinates
        let cx = nx - 0.5;
        let cy = ny - 0.5;

        // Two overlapping circular patterns with different centers
        let offset = self.moire_offset;
        let c1x = 0.1 * offset.cos();
        let c1y = 0.1 * offset.sin();
        let c2x = -0.1 * offset.cos();
        let c2y = -0.1 * offset.sin();

        let r1 = ((cx - c1x).powi(2) + (cy - c1y).powi(2)).sqrt();
        let r2 = ((cx - c2x).powi(2) + (cy - c2y).powi(2)).sqrt();

        // Create interference pattern
        let freq = 30.0 + (self.time * 0.5).sin() * 10.0;
        let pattern1 = (r1 * freq).sin();
        let pattern2 = (r2 * freq).sin();

        // Moiré effect from overlapping patterns
        let moire = (pattern1 + pattern2) / 2.0;
        let normalized = (moire + 1.0) / 2.0;

        let hue = (self.base_hue + self.hue_offset + normalized * 60.0) % 360.0;
        let saturation = 0.6 + normalized * 0.3;
        let value = 0.3 + normalized * 0.6;

        Color::from_hsv(hue, saturation, value)
    }

    /// Render the current pattern to a raw RGB buffer
    fn render_to_buffer(&self, buffer: &mut [u8]) {
        let height = self.config.height as usize;
        let width = self.config.width as usize;
        let blocks_x = self.config.blocks_x() as usize;
        let blocks_y = self.config.blocks_y() as usize;

        for block_y in 0..blocks_y {
            for block_x in 0..blocks_x {
                let block_size = self.get_effective_block_size(block_x, block_y);
                let color = self.get_block_color(block_x, block_y, blocks_x, blocks_y);

                let start_y = block_y * self.config.block_size as usize;
                let start_x = block_x * self.config.block_size as usize;
                let end_y = (start_y + block_size).min(height);
                let end_x = (start_x + block_size).min(width);

                for y in start_y..end_y {
                    for x in start_x..end_x {
                        let idx = (y * width + x) * 3;
                        if idx + 2 < buffer.len() {
                            buffer[idx] = color.r;
                            buffer[idx + 1] = color.g;
                            buffer[idx + 2] = color.b;
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mosaic_config() {
        let config = MosaicConfig::new(1280, 720, 64, 0.5, 24);
        assert_eq!(config.blocks_x(), 20);
        assert_eq!(config.blocks_y(), 12);
        assert_eq!(config.frames_per_change(), 12);
    }

    #[test]
    fn test_color_from_hsv() {
        let red = Color::from_hsv(0.0, 1.0, 1.0);
        assert_eq!(red.r, 255);
        assert!(red.g < 10);
        assert!(red.b < 10);

        let green = Color::from_hsv(120.0, 1.0, 1.0);
        assert!(green.r < 10);
        assert_eq!(green.g, 255);
        assert!(green.b < 10);
    }

    #[test]
    fn test_pattern_modes() {
        assert_eq!(PatternMode::from_str("perlin"), Some(PatternMode::Perlin));
        assert_eq!(PatternMode::from_str("PLASMA"), Some(PatternMode::Plasma));
        assert_eq!(PatternMode::from_str("fbm"), Some(PatternMode::Fbm));
        assert_eq!(PatternMode::from_str("warp"), Some(PatternMode::Warp));
        assert_eq!(PatternMode::from_str("lissajous"), Some(PatternMode::Lissajous));
        assert_eq!(PatternMode::from_str("kaleido"), Some(PatternMode::Kaleidoscope));
        assert_eq!(PatternMode::from_str("spiral"), Some(PatternMode::Spiral));
        assert_eq!(PatternMode::from_str("checker"), Some(PatternMode::Checker));
        assert_eq!(PatternMode::from_str("walker"), Some(PatternMode::Walker));
        assert_eq!(PatternMode::from_str("human"), Some(PatternMode::Walker));
        assert_eq!(PatternMode::from_str("metaballs"), Some(PatternMode::Metaballs));
        assert_eq!(PatternMode::from_str("blobs"), Some(PatternMode::Metaballs));
        assert_eq!(PatternMode::from_str("julia"), Some(PatternMode::Julia));
        assert_eq!(PatternMode::from_str("fractal"), Some(PatternMode::Julia));
        assert_eq!(PatternMode::from_str("truchet"), Some(PatternMode::Truchet));
        assert_eq!(PatternMode::from_str("maze"), Some(PatternMode::Truchet));
        assert_eq!(PatternMode::from_str("matrix"), Some(PatternMode::Matrix));
        assert_eq!(PatternMode::from_str("rain"), Some(PatternMode::Matrix));
        assert_eq!(PatternMode::from_str("fire"), Some(PatternMode::Fire));
        assert_eq!(PatternMode::from_str("moire"), Some(PatternMode::Moire));
        assert_eq!(PatternMode::from_str("random"), Some(PatternMode::Random));
        assert_eq!(PatternMode::from_str("invalid"), None);
    }

    #[test]
    fn test_mosaic_generator_buffer() {
        let config = MosaicConfig::new(100, 80, 20, 1.0, 24);
        let mut generator = MosaicGenerator::new(config);
        let mut buffer = vec![0u8; 100 * 80 * 3];
        generator.write_frame_to_buffer(&mut buffer);

        let non_zero = buffer.iter().any(|&b| b != 0);
        assert!(non_zero);
    }

    #[test]
    fn test_all_pattern_modes() {
        for mode in PatternMode::all_modes() {
            let config = MosaicConfig::new(100, 80, 20, 1.0, 24)
                .with_pattern_mode(*mode);
            let mut generator = MosaicGenerator::new(config);
            let mut buffer = vec![0u8; 100 * 80 * 3];
            generator.write_frame_to_buffer(&mut buffer);

            let non_zero = buffer.iter().any(|&b| b != 0);
            assert!(non_zero, "Pattern {:?} should produce non-zero output", mode);
        }
    }
}
