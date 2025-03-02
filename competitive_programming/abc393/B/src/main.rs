use proconio::input;

fn main() {
    input! {
        s: String
    }

    let chars: Vec<char> = s.chars().collect();
    let n = chars.len();
    let mut count = 0;

    // 各位置の組み合わせを試す
    for i in 0..n - 2 {
        for j in i + 1..n - 1 {
            let k = j + (j - i); // 等間隔の条件より
            if k >= n {
                continue;
            }

            // A,B,Cがこの順に並んでいるかチェック
            if chars[i] == 'A' && chars[j] == 'B' && chars[k] == 'C' {
                count += 1;
            }
        }
    }

    println!("{}", count);
}
