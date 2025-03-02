use proconio::input;

fn main() {
    input! {
        n: usize,
    }

    // グリッドを初期化（最初は何も塗られていない状態としてダミー文字を使用）
    let mut grid = vec![vec!['?'; n]; n];

    // i=1,2,...,N の順に操作を行う
    for i in 1..=n {
        let j = n + 1 - i;

        // i≤j であるときだけ塗りつぶす
        if i <= j {
            // 色を決定（i が奇数なら黒('#')、偶数なら白('.')）
            let color = if i % 2 == 1 { '#' } else { '.' };

            // マス(i,i)を左上、マス(j,j)を右下とする矩形領域を塗りつぶす
            // 0-indexedに調整するため、i-1, j-1を使用
            for row in (i - 1)..j {
                for col in (i - 1)..j {
                    grid[row][col] = color;
                }
            }
        }
    }

    // 結果を出力
    for row in &grid {
        for &cell in row {
            // 万が一未塗装のマスがあれば確認できるように
            if cell == '?' {
                panic!("塗られていないマスが存在します。");
            }
            print!("{}", cell);
        }
        println!();
    }
}
