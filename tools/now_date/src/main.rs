use chrono::Local;

fn main() {
    let today = Local::now().format("%Y-%m-%d").to_string();
    println!("Hello, world! 今日の日付は {} です。", today);
}
