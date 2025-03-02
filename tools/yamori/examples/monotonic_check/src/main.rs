use proconio::input;

fn main() {
    input! {
        n: usize,
        a: [i32; n],
    }

    // 狭義単調増加かどうかを判定
    let mut is_strictly_increasing = true;

    for i in 0..n-1 {
        if a[i] >= a[i+1] {
            is_strictly_increasing = false;
            break;
        }
    }

    if is_strictly_increasing {
        println!("Yes");
    } else {
        println!("No");
    }
} 