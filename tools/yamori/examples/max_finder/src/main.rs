use proconio::input;

fn main() {
    input! {
        n: usize,
        a: [i32; n],
    }

    // 配列の最大値を求める
    let max_value = a.iter().max().unwrap_or(&0);
    
    println!("{}", max_value);
} 