use proconio::input;

fn main() {
    input! {
        n: usize,
        a: [i32; n],
    }

    // 偶数の数をカウント
    let even_count = a.iter().filter(|&x| x % 2 == 0).count();
    
    println!("{}", even_count);
} 