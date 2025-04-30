use std::fs;
use std::path::Path;
use std::thread;
use std::time::{Duration, Instant, SystemTime};
use xcap::Monitor;

// スクリーンショット撮影の設定
const SCREENSHOT_INTERVAL: u64 = 5; // 5秒ごとにスクリーンショットを撮影
const SAVE_PATH: &str = "slack_screenshots";

// 簡易なSlackウィンドウ検出機能
// この関数はスクリーンショットのRGBAピクセルデータから紫色のピクセルの割合を計算し、
// Slackウィンドウが表示されているかどうかを判断します
fn detect_slack_window(rgba_data: &[u8], width: u32, height: u32) -> bool {
    // Slackの紫色の範囲（RGB値）
    let purple_lower_r = 100;
    let purple_lower_g = 50;
    let purple_lower_b = 130;

    let purple_upper_r = 170;
    let purple_upper_g = 100;
    let purple_upper_b = 210;

    let mut purple_pixel_count = 0;
    let total_pixels = (width * height) as usize;

    // ピクセルデータを4バイトずつ処理（RGBA）
    for i in (0..rgba_data.len()).step_by(4) {
        if i + 2 < rgba_data.len() {
            let r = rgba_data[i];
            let g = rgba_data[i + 1];
            let b = rgba_data[i + 2];

            // 指定した範囲内の紫色かどうかを判定
            if r >= purple_lower_r
                && r <= purple_upper_r
                && g >= purple_lower_g
                && g <= purple_upper_g
                && b >= purple_lower_b
                && b <= purple_upper_b
            {
                purple_pixel_count += 1;
            }
        }
    }

    // 閾値: 紫色のピクセルが一定数以上あればSlackウィンドウと判断
    let threshold_ratio = 0.001; // 全ピクセルの0.1%以上が紫色
    let has_enough_purple = (purple_pixel_count as f64 / total_pixels as f64) > threshold_ratio;

    // デバッグ用（閾値調整に便利）
    println!(
        "紫色ピクセル数: {}, 全ピクセル数: {}, 比率: {:.6}",
        purple_pixel_count,
        total_pixels,
        purple_pixel_count as f64 / total_pixels as f64
    );

    has_enough_purple
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 保存用ディレクトリの作成
    if !Path::new(SAVE_PATH).exists() {
        fs::create_dir(SAVE_PATH)?;
    }

    println!("Slackスクリーンショットモニタリングを開始しました");
    println!(
        "スクリーンショットは{}ディレクトリに保存されます",
        SAVE_PATH
    );
    println!("終了するには Ctrl+C を押してください");

    let mut last_saved_time = Instant::now() - Duration::from_secs(SCREENSHOT_INTERVAL);
    let mut screenshot_count = 0;

    // メインループ
    loop {
        let current_time = Instant::now();

        // 指定した間隔が経過したらスクリーンショットを撮影
        if current_time.duration_since(last_saved_time).as_secs() >= SCREENSHOT_INTERVAL {
            last_saved_time = current_time;

            // すべてのモニターを取得
            let monitors = Monitor::all()?;
            let primary_monitor = monitors
                .iter()
                .find(|m| m.is_primary().unwrap_or(false))
                .unwrap_or(&monitors[0]);

            // スクリーンショットを撮影
            let image = primary_monitor.capture_image()?;
            let width = image.width();
            let height = image.height();

            // XCapのImageからRGBAデータを取得
            let rgba_data = image.as_raw();

            // Slackウィンドウの検出
            if detect_slack_window(rgba_data, width, height) {
                // スクリーンショットを保存
                let timestamp = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)?
                    .as_secs();
                let filename = format!("{}/slack_screenshot_{}.png", SAVE_PATH, timestamp);

                // XCapのsaveメソッドを使用して直接保存
                image.save(&filename)?;

                println!(
                    "Slackウィンドウを検出しました。スクリーンショットを保存: {}",
                    filename
                );
                screenshot_count += 1;
            }
        }

        // CPUの負荷を下げるためのスリープ
        thread::sleep(Duration::from_millis(500));
    }
}
