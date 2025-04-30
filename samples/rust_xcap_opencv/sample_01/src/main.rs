use opencv::core::{CV_8UC4, Mat, Size};
use opencv::highgui;
use opencv::imgproc;
use opencv::prelude::*;
use std::time::Instant;
use xcap::Monitor;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // OpenCVのウィンドウを作成
    highgui::named_window("Screenshot", highgui::WINDOW_AUTOSIZE)?;
    highgui::named_window("Processed", highgui::WINDOW_AUTOSIZE)?;

    println!("Press 'q' to exit");

    // メインループ
    loop {
        let start = Instant::now();

        // プライマリモニターを取得
        let monitors = Monitor::all()?;
        let primary_monitor = monitors
            .iter()
            .find(|m| m.is_primary().unwrap_or(false))
            .unwrap_or(&monitors[0]);

        // スクリーンショットを撮影
        let image = primary_monitor.capture_image()?;
        let width = image.width() as i32;
        let height = image.height() as i32;

        // ピクセルデータを取得
        let raw_pixels = image.as_raw();

        // OpenCVのMat形式に変換
        let mut mat = unsafe {
            let mut mat = Mat::new_size(Size::new(width, height), CV_8UC4)?;

            // データをコピー
            let mat_data = mat.data_mut();
            std::ptr::copy_nonoverlapping(
                raw_pixels.as_ptr(),
                mat_data,
                (width * height * 4) as usize,
            );

            mat
        };

        // 元のスクリーンショットを表示
        highgui::imshow("Screenshot", &mat)?;

        // 画像処理の例: グレースケール変換
        let mut gray = Mat::default();
        imgproc::cvt_color(
            &mat,
            &mut gray,
            imgproc::COLOR_BGRA2GRAY,
            0,
            opencv::core::AlgorithmHint::ALGO_HINT_DEFAULT,
        )?;

        // エッジ検出の例
        let mut edges = Mat::default();
        imgproc::canny(&gray, &mut edges, 100.0, 200.0, 3, false)?;

        // 処理した画像を表示
        highgui::imshow("Processed", &edges)?;

        // 処理時間を表示
        println!("処理時間: {:?}", start.elapsed());

        // キー入力を待つ（10ms）
        let key = highgui::wait_key(10)?;
        if key == 'q' as i32 || key == 'Q' as i32 {
            break;
        }
    }

    Ok(())
}
