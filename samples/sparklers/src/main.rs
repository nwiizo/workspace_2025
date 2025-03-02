use colored::*;
use rand::Rng;
use std::{thread, time::Duration};

struct Senko {
    base_x: f64,          // 紐の位置 (x-coordinate of the string)
    base_y: f64,          // 紐の長さ (length of the string)
    fire_y: f64,          // 火元の現在位置 (current position of the fire source)
    sparks: Vec<Spark>,   // 火花のベクター (vector of sparks)
    lifetime: i32,        // 線香花火の寿命 (lifetime of the Senko Hanabi)
    string_length: usize, // 紐の長さ (length of the string)
    falling: bool,        // 線香花火が落ちているかどうか (whether the Senko Hanabi is falling)
    fall_speed: f64,      // 落下速度 (falling speed)
}

struct Spark {
    x: f64,           // 火花の現在のx座標
    y: f64,           // 火花の現在のy座標
    vx: f64,          // 火花のx方向の速度
    vy: f64,          // 火花のy方向の速度
    lifetime: i32,    // 火花の寿命（フレーム数）
    char: char,       // 火花を表す文字
    temperature: f64, // 火花の温度（0.0（冷たい）から1.0（熱い）まで）
}

impl Senko {
    fn new(x: f64) -> Self {
        let string_length = 8;
        Senko {
            base_x: x,
            base_y: 1.0,
            fire_y: (string_length + 1) as f64,
            sparks: Vec::new(),
            lifetime: rand::thread_rng().gen_range(300..400),
            string_length,
            falling: false,
            fall_speed: 0.0,
        }
    }

    fn update(&mut self) {
        if self.falling {
            self.fall_speed += 0.1;
            self.fire_y += self.fall_speed;
        }

        // 火花生成のロジック
        if !self.falling && self.lifetime > 0 {
            // メインの火花生成
            if rand::thread_rng().gen_bool(0.6) {
                let angle: f64 = rand::thread_rng().gen_range(0.0..std::f64::consts::PI * 2.0);
                let speed: f64 = rand::thread_rng().gen_range(0.1..0.3);
                let temp: f64 = rand::thread_rng().gen_range(0.8..1.0); // 高温の火花

                self.sparks.push(Spark {
                    x: self.base_x,
                    y: self.fire_y,
                    vx: angle.cos() * speed,
                    vy: angle.sin() * speed,
                    lifetime: rand::thread_rng().gen_range(20..30),
                    char: '✺',
                    temperature: temp,
                });
            }

            // 火元周りの熱光エフェクト
            if rand::thread_rng().gen_bool(0.5) {
                let tiny_angle: f64 = rand::thread_rng().gen_range(0.0..std::f64::consts::PI * 2.0);
                let tiny_speed: f64 = rand::thread_rng().gen_range(0.05..0.15);
                self.sparks.push(Spark {
                    x: self.base_x,
                    y: self.fire_y,
                    vx: tiny_angle.cos() * tiny_speed,
                    vy: tiny_angle.sin() * tiny_speed,
                    lifetime: rand::thread_rng().gen_range(5..12),
                    char: '･',
                    temperature: 1.0, // 最高温
                });
            }
        }

        // 火花の更新と分裂
        let mut new_sparks = Vec::new();
        for spark in &mut self.sparks {
            spark.update();

            if spark.temperature > 0.8 && rand::thread_rng().gen_bool(0.15) {
                let num_children = rand::thread_rng().gen_range(3..6);
                for _ in 0..num_children {
                    let angle: f64 = rand::thread_rng().gen_range(0.0..std::f64::consts::PI * 2.0);
                    let child_speed: f64 = rand::thread_rng().gen_range(0.05..0.15);
                    let child_temp = spark.temperature * 0.8; // 子火花は少し冷める
                    new_sparks.push(Spark {
                        x: spark.x,
                        y: spark.y,
                        vx: angle.cos() * child_speed,
                        vy: angle.sin() * child_speed,
                        lifetime: rand::thread_rng().gen_range(8..15),
                        char: '･',
                        temperature: child_temp,
                    });
                }
            }
        }
        self.sparks.extend(new_sparks);
        self.sparks.retain(|s| s.lifetime > 0);

        self.lifetime -= 1;
        if self.lifetime <= 0 && !self.falling {
            self.falling = true;
        }
    }

    fn is_done(&self) -> bool {
        self.falling && self.fire_y > 24.0
    }
}

impl Spark {
    fn update(&mut self) {
        self.x += self.vx;
        self.y += self.vy;
        self.vy += 0.015;
        self.lifetime -= 1;
        self.temperature *= 0.98; // 徐々に冷める
    }
}

fn get_temp_color(temp: f64, ch: char) -> ColoredString {
    if temp > 0.9 {
        ch.to_string().bright_white() // 白熱
    } else if temp > 0.7 {
        ch.to_string().bright_yellow() // 黄色い熱
    } else if temp > 0.5 {
        ch.to_string().bright_red() // 赤熱
    } else if temp > 0.3 {
        ch.to_string().red() // 暗い赤
    } else {
        ch.to_string().yellow() // 残り火
    }
}

fn draw_frame(senko: &Senko) {
    clear_screen();
    let mut frame = vec![vec![(' ', 0.0); 80]; 24];

    // まっすぐな糸を描画
    for i in 0..senko.string_length {
        let y = (senko.base_y + i as f64) as usize;
        if y < frame.len() {
            frame[y][senko.base_x as usize] = ('│', 0.0);
        }
    }

    // 火元の描画
    let fire_x = senko.base_x as usize;
    let fire_y = senko.fire_y as usize;
    if fire_y < frame.len() && fire_x < frame[0].len() {
        // 火元周辺の熱グロー
        for dy in -2..=2 {
            for dx in -2..=2 {
                let nx = (fire_x as i32 + dx) as usize;
                let ny = (fire_y as i32 + dy) as usize;
                if ny < frame.len() && nx < frame[0].len() {
                    let dist = (dx * dx + dy * dy) as f64;
                    if dist <= 2.0 && frame[ny][nx].0 == ' ' {
                        frame[ny][nx] = ('⋅', (3.0 - dist) / 3.0);
                    }
                }
            }
        }
        frame[fire_y][fire_x] = ('◉', 1.0);
    }

    // 火花の描画
    for spark in &senko.sparks {
        let x = spark.x as usize;
        let y = spark.y as usize;
        if y < frame.len() && x < frame[0].len() {
            frame[y][x] = (spark.char, spark.temperature);
        }
    }

    // フレームの描画
    for row in frame {
        for (ch, temp) in row {
            if ch == ' ' {
                print!(" ");
            } else if ch == '│' {
                print!("{}", ch.to_string().white());
            } else {
                print!("{}", get_temp_color(temp, ch));
            }
        }
        println!();
    }
}

fn clear_screen() {
    print!("\x1B[2J\x1B[1;1H");
}

fn main() {
    // 線香花火のシミュレーションを開始
    println!("線香花火をお楽しみください...");
    thread::sleep(Duration::from_secs(2));
    clear_screen();

    // 線香花火の初期位置を設定
    let mut senko = Senko::new(40.0);

    // 線香花火が終了するまでループ
    while !senko.is_done() {
        senko.update(); // 線香花火の状態を更新
        draw_frame(&senko); // 現在のフレームを描画
        thread::sleep(Duration::from_millis(50)); // 少し待機して次のフレームへ
    }
}
