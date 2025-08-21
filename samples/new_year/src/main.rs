use chrono::{Datelike, Local};
use colored::*;
use rand::Rng;
use std::thread::sleep;
use std::{thread, time::Duration};

struct Firework {
    x: f64,
    y: f64,
    velocity: f64,
    particles: Vec<Particle>,
    exploded: bool,
    color: Color,
    sparkles: Vec<Sparkle>,
}

struct Sparkle {
    x: f64,
    y: f64,
    lifetime: i32,
}

#[derive(Clone, Copy)]
enum Color {
    Red,
    Green,
    Blue,
    Yellow,
    Magenta,
    Cyan,
    Rainbow,
    Silver,
    Gold,
    Pearl,
}

impl Color {
    fn random() -> Self {
        let colors = [
            Color::Red,
            Color::Green,
            Color::Blue,
            Color::Yellow,
            Color::Magenta,
            Color::Cyan,
            Color::Rainbow,
            Color::Silver,
            Color::Gold,
            Color::Pearl,
        ];
        colors[rand::thread_rng().gen_range(0..colors.len())]
    }

    fn get_colored_char(&self, c: char, time: u32) -> colored::ColoredString {
        match self {
            Color::Red => c.to_string().bright_red(),
            Color::Green => c.to_string().bright_green(),
            Color::Blue => c.to_string().bright_blue(),
            Color::Yellow => c.to_string().bright_yellow(),
            Color::Magenta => c.to_string().bright_magenta(),
            Color::Cyan => c.to_string().bright_cyan(),
            Color::Silver => {
                if time % 2 == 0 {
                    c.to_string().white()
                } else {
                    c.to_string().bright_white()
                }
            }
            Color::Gold => {
                if time % 2 == 0 {
                    c.to_string().yellow()
                } else {
                    c.to_string().bright_yellow()
                }
            }
            Color::Pearl => match (time / 3) % 3 {
                0 => c.to_string().bright_white(),
                1 => c.to_string().bright_cyan(),
                _ => c.to_string().white(),
            },
            Color::Rainbow => match (time / 4) % 7 {
                0 => c.to_string().bright_red(),
                1 => c.to_string().bright_yellow(),
                2 => c.to_string().bright_green(),
                3 => c.to_string().bright_cyan(),
                4 => c.to_string().bright_blue(),
                5 => c.to_string().bright_magenta(),
                _ => c.to_string().bright_white(),
            },
        }
    }
}

struct Particle {
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    lifetime: i32,
    char: char,
    color: Color,
    trail: Vec<(f64, f64)>,
}

impl Firework {
    fn new(x: f64) -> Self {
        Firework {
            x,
            y: 25.0,
            velocity: rand::thread_rng().gen_range(0.6..1.1),
            particles: Vec::new(),
            exploded: false,
            color: Color::random(),
            sparkles: Vec::new(),
        }
    }

    fn update(&mut self) {
        if !self.exploded {
            self.y -= self.velocity;
            if rand::thread_rng().gen_bool(0.3) {
                self.sparkles.push(Sparkle {
                    x: self.x + rand::thread_rng().gen_range(-0.5..0.5),
                    y: self.y + rand::thread_rng().gen_range(-0.5..0.5),
                    lifetime: 5,
                });
            }
            if self.y <= rand::thread_rng().gen_range(5.0..15.0) {
                self.explode();
            }
        } else {
            for particle in &mut self.particles {
                particle.update();
            }
            self.particles.retain(|p| p.lifetime > 0);
        }

        for sparkle in &mut self.sparkles {
            sparkle.lifetime -= 1;
        }
        self.sparkles.retain(|s| s.lifetime > 0);
    }

    fn explode(&mut self) {
        self.exploded = true;
        let num_particles = rand::thread_rng().gen_range(35..55);
        let particle_chars = ['✦', '✴', '⋆', '✳', '✷', '❈', '✺', '✹', '✸', '✶'];

        for _ in 0..num_particles {
            let angle = rand::thread_rng().gen_range(0.0..std::f64::consts::PI * 2.0);
            let speed = rand::thread_rng().gen_range(0.2..0.6);
            let char_idx = rand::thread_rng().gen_range(0..particle_chars.len());

            self.particles.push(Particle {
                x: self.x,
                y: self.y,
                vx: angle.cos() * speed,
                vy: angle.sin() * speed,
                lifetime: rand::thread_rng().gen_range(30..45),
                char: particle_chars[char_idx],
                color: if matches!(self.color, Color::Rainbow) {
                    Color::random()
                } else {
                    self.color
                },
                trail: Vec::new(),
            });
        }
    }

    fn is_done(&self) -> bool {
        self.exploded && self.particles.is_empty()
    }
}

impl Particle {
    fn update(&mut self) {
        self.trail.push((self.x, self.y));
        if self.trail.len() > 5 {
            self.trail.remove(0);
        }

        self.x += self.vx;
        self.y += self.vy;
        self.vy += 0.015; // reduced gravity for more floating effect
        self.lifetime -= 1;
    }
}

fn clear_screen() {
    print!("\x1B[2J\x1B[1;1H");
}

fn draw_frame(fireworks: &Vec<Firework>, frame_count: u32) {
    clear_screen();
    let mut frame = vec![vec![(' ', None); 100]; 30];

    // Add background stars
    if frame_count % 10 == 0 {
        for _ in 0..50 {
            let x = rand::thread_rng().gen_range(0..100);
            let y = rand::thread_rng().gen_range(0..30);
            frame[y][x] = ('·', Some(Color::Silver));
        }
    }

    for firework in fireworks {
        // Draw launch sparkles
        for sparkle in &firework.sparkles {
            let x = sparkle.x as usize;
            let y = sparkle.y as usize;
            if y < frame.len() && x < frame[0].len() {
                frame[y][x] = ('｡', Some(Color::Pearl));
            }
        }

        if !firework.exploded {
            let x = firework.x as usize;
            let y = firework.y as usize;
            if y < frame.len() && x < frame[0].len() {
                frame[y][x] = ('⁂', Some(firework.color));
            }
        } else {
            for particle in &firework.particles {
                // Draw particle trails
                for (i, (trail_x, trail_y)) in particle.trail.iter().enumerate() {
                    let x = *trail_x as usize;
                    let y = *trail_y as usize;
                    if y < frame.len() && x < frame[0].len() {
                        let trail_char = match i {
                            0 => '.',
                            1 => '·',
                            _ => '°',
                        };
                        frame[y][x] = (trail_char, Some(particle.color));
                    }
                }

                let x = particle.x as usize;
                let y = particle.y as usize;
                if y < frame.len() && x < frame[0].len() {
                    frame[y][x] = (particle.char, Some(particle.color));
                }
            }
        }
    }

    for row in frame {
        for (char, color) in row {
            match color {
                Some(c) => print!("{}", c.get_colored_char(char, frame_count)),
                None => print!(" "),
            }
        }
        println!();
    }
}

fn display_big_text(year: i32, color: Color) {
    let happy = vec![
        "██╗  ██╗ █████╗ ██████╗ ██████╗ ██╗   ██╗",
        "██║  ██║██╔══██╗██╔══██╗██╔══██╗╚██╗ ██╔╝",
        "███████║███████║██████╔╝██████╔╝ ╚████╔╝ ",
        "██╔══██║██╔══██║██╔═══╝ ██╔═══╝   ╚██╔╝  ",
        "██║  ██║██║  ██║██║     ██║        ██║   ",
        "╚═╝  ╚═╝╚═╝  ╚═╝╚═╝     ╚═╝        ╚═╝   ",
    ];

    let new = vec![
        "███╗   ██╗███████╗██╗    ██╗",
        "████╗  ██║██╔════╝██║    ██║",
        "██╔██╗ ██║█████╗  ██║ █╗ ██║",
        "██║╚██╗██║██╔══╝  ██║███╗██║",
        "██║ ╚████║███████╗╚███╔███╔╝",
        "╚═╝  ╚═══╝╚══════╝ ╚══╝╚══╝ ",
    ];

    let year_text = vec![
        "██╗   ██╗███████╗ █████╗ ██████╗ ",
        "╚██╗ ██╔╝██╔════╝██╔══██╗██╔══██╗",
        " ╚████╔╝ █████╗  ███████║██████╔╝",
        "  ╚██╔╝  ██╔══╝  ██╔══██║██╔══██╗",
        "   ██║   ███████╗██║  ██║██║  ██║",
        "   ╚═╝   ╚══════╝╚═╝  ╚═╝╚═╝  ╚═╝",
    ];

    let digit_art = [
        // 0
        vec![
            "██████╗ ",
            "██╔══██╗",
            "██║  ██║",
            "██║  ██║",
            "╚█████╔╝",
            " ╚════╝ ",
        ],
        // 1
        vec![" ██╗", "███║", "╚██║", " ██║", " ██║", " ╚═╝"],
        // 2
        vec![
            "██████╗ ",
            "╚════██╗",
            " █████╔╝",
            "██╔═══╝ ",
            "███████╗",
            "╚══════╝",
        ],
        // 3
        vec![
            "██████╗ ",
            "╚════██╗",
            " █████╔╝",
            " ╚═══██╗",
            "██████╔╝",
            "╚═════╝ ",
        ],
        // 4
        vec![
            "██╗  ██╗",
            "██║  ██║",
            "███████║",
            "╚════██║",
            "     ██║",
            "     ╚═╝",
        ],
        // 5
        vec![
            "██████╗ ",
            "██╔═══╝ ",
            "██████╗ ",
            "╚════██╗",
            "██████╔╝",
            "╚═════╝ ",
        ],
        // 6
        vec![
            " █████╗ ",
            "██╔═══╝ ",
            "██████╗ ",
            "██╔══██╗",
            "╚█████╔╝",
            " ╚════╝ ",
        ],
        // 7
        vec![
            "██████╗ ",
            "╚════██╗",
            "    ██╔╝",
            "   ██╔╝ ",
            "   ██║  ",
            "   ╚═╝  ",
        ],
        // 8
        vec![
            " █████╗ ",
            "██╔══██╗",
            "╚█████╔╝",
            "██╔══██╗",
            "╚█████╔╝",
            " ╚════╝ ",
        ],
        // 9
        vec![
            " █████╗ ",
            "██╔══██╗",
            "╚██████║",
            " ╚═══██║",
            " █████╔╝",
            " ╚════╝ ",
        ],
    ];

    let year_digits: Vec<char> = year.to_string().chars().collect();
    let mut year_lines = vec!["".to_string(); 6];
    for i in 0..6 {
        for &digit in &year_digits {
            let digit_idx = digit.to_digit(10).unwrap() as usize;
            year_lines[i].push_str(digit_art[digit_idx][i]);
            year_lines[i].push(' ');
        }
    }

    let year_art: Vec<&str> = year_lines.iter().map(|s| s.as_str()).collect();
    let words = [&happy[..], &new[..], &year_text[..], &year_art[..]];
    let mut frame_count = 0;

    for i in 0..6 {
        for word in &words {
            sleep(Duration::from_millis(100));
            print!("{} ", color.get_colored_char('✧', frame_count).to_string());
            print!("{}", word[i]);
            print!("{} ", color.get_colored_char('✧', frame_count).to_string());
        }
        println!();
        frame_count += 1;
    }
}

fn display_new_year_message(year: i32) {
    println!("\n\n");
    display_big_text(year, Color::Rainbow);
    println!("\n");

    let messages = [
        "✧･ﾟ: *✧･ﾟ:* May your dreams shimmer with stardust *:･ﾟ✧*:･ﾟ✧",
        "*.✦   Let your spirit dance with the northern lights   ✦.*",
        "✵°•  Each moment a precious crystal of time  •°✵",
        "⋆｡ﾟ✶° Embrace the magic of new beginnings °✶ﾟ｡⋆",
    ];

    for (i, msg) in messages.iter().enumerate() {
        let color = match i {
            0 => Color::Pearl,
            1 => Color::Silver,
            2 => Color::Gold,
            _ => Color::Rainbow,
        };
        println!(
            "{}",
            color.get_colored_char('⋆', i as u32).to_string().repeat(3)
        );
        println!("{}", msg.bright_white());
        println!(
            "{}",
            color.get_colored_char('⋆', i as u32).to_string().repeat(3)
        );
        thread::sleep(Duration::from_millis(300));
    }
    println!("\n");
}

fn main() {
    // Get current year
    let year = Local::now().year();

    let mut fireworks = Vec::new();
    let mut frame_count = 0;

    loop {
        if frame_count % 15 == 0 && fireworks.len() < 8 {
            fireworks.push(Firework::new(rand::thread_rng().gen_range(10.0..90.0)));
        }

        for firework in &mut fireworks {
            firework.update();
        }

        draw_frame(&fireworks, frame_count);
        fireworks.retain(|f| !f.is_done());

        thread::sleep(Duration::from_millis(40));
        frame_count += 1;

        if frame_count > 300 {
            clear_screen();
            display_new_year_message(year);
            break;
        }
    }
}
