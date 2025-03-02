use proconio::input;

fn main() {
    input! {
        n: usize,
        a: [i64; n],
    }

    let mut answer = 0;

    // 上になる餅を固定
    for top in 0..n {
        // topより大きな位置で、条件を満たす最初の餅を二分探索
        let mut left = 0;
        let mut right = n;

        while left < right {
            let mid = (left + right) / 2;
            if a[top] * 2 <= a[mid] {
                right = mid;
            } else {
                left = mid + 1;
            }
        }

        // left以降の餅は全て条件を満たす
        if left < n {
            // top自身は除外
            if left <= top {
                answer += n - left - 1;
            } else {
                answer += n - left;
            }
        }
    }

    println!("{}", answer);
}
