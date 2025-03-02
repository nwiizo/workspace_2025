use proconio::input;
use std::cmp::{max, min};
use std::collections::HashSet;

fn solve(n: usize, x: i64, teeth: Vec<(i64, i64)>) -> i64 {
    // 単一の歯のケース
    if n == 1 {
        return 0; // 1本だけなら調整の必要なし
    }

    // 上の歯と下の歯を取得
    let upper: Vec<i64> = teeth.iter().map(|&(u, _)| u).collect();
    let lower: Vec<i64> = teeth.iter().map(|&(_, d)| d).collect();

    // 可能なHの値の候補を収集
    let mut h_values = HashSet::new();

    // 各位置での現在の上下の和を候補に追加
    for i in 0..n {
        h_values.insert(upper[i] + lower[i]);
    }

    // すべての上下歯の組合せを候補に
    for i in 0..n {
        for j in 0..n {
            h_values.insert(upper[i] + lower[j]);
        }
    }

    // 候補をベクトルに変換してソート
    let mut h_values_vec: Vec<i64> = h_values.into_iter().collect();
    h_values_vec.sort_unstable();

    let mut min_cost = i64::MAX;

    // 各Hの候補について検証
    'h_loop: for &h in &h_values_vec {
        // 上の歯の可能な高さ範囲
        let mut upper_bounds = vec![0; n];
        let mut lower_bounds = vec![0; n];

        // 初期範囲を設定
        for i in 0..n {
            // 上限: 元の歯の高さ以下
            upper_bounds[i] = upper[i];
            // 下限: 下の歯がH以下なら必要な分確保、そうでなければ0でも良い
            lower_bounds[i] = if lower[i] <= h {
                max(0, h - lower[i])
            } else {
                0
            };

            // 範囲の整合性チェック
            if lower_bounds[i] > upper_bounds[i] {
                continue 'h_loop; // この候補は不可能
            }
        }

        // 隣接制約を反映するために繰り返し伝播
        let mut changed = true;
        while changed {
            changed = false;

            // 左から右への伝播
            for i in 0..n - 1 {
                // i番目の下限から、i+1番目の下限を更新
                let new_lower = max(lower_bounds[i + 1], lower_bounds[i] - x);
                if new_lower > lower_bounds[i + 1] {
                    if new_lower > upper_bounds[i + 1] {
                        continue 'h_loop; // 不可能
                    }
                    lower_bounds[i + 1] = new_lower;
                    changed = true;
                }

                // i+1番目の下限から、i番目の下限を更新
                let new_lower = max(lower_bounds[i], lower_bounds[i + 1] - x);
                if new_lower > lower_bounds[i] {
                    if new_lower > upper_bounds[i] {
                        continue 'h_loop; // 不可能
                    }
                    lower_bounds[i] = new_lower;
                    changed = true;
                }
            }

            // 右から左への伝播
            for i in (0..n - 1).rev() {
                // i+1番目の上限から、i番目の上限を更新
                let new_upper = min(upper_bounds[i], upper_bounds[i + 1] + x);
                if new_upper < upper_bounds[i] {
                    if new_upper < lower_bounds[i] {
                        continue 'h_loop; // 不可能
                    }
                    upper_bounds[i] = new_upper;
                    changed = true;
                }

                // i番目の上限から、i+1番目の上限を更新
                let new_upper = min(upper_bounds[i + 1], upper_bounds[i] + x);
                if new_upper < upper_bounds[i + 1] {
                    if new_upper < lower_bounds[i + 1] {
                        continue 'h_loop; // 不可能
                    }
                    upper_bounds[i + 1] = new_upper;
                    changed = true;
                }
            }
        }

        // コスト計算
        let mut cost = 0;
        for i in 0..n {
            // まずこの位置について問題が解けるか確認
            if lower_bounds[i] > upper_bounds[i] {
                continue 'h_loop; // この候補は不可能
            }

            // 上の歯の最適な高さ（元の高さに最も近いもの）
            let optimal_upper = max(lower_bounds[i], min(upper_bounds[i], upper[i]));

            // 下の歯の必要な高さ
            let required_lower = h - optimal_upper;

            // 上の歯と下の歯それぞれの削る量を計算
            let upper_cut = max(0, upper[i] - optimal_upper);
            let lower_cut = max(0, lower[i] - required_lower);

            cost += upper_cut + lower_cut;
        }

        min_cost = min(min_cost, cost);
    }

    // 解が見つからなかった場合は0を返す（すでにうまく噛み合っている）
    if min_cost == i64::MAX {
        return 0;
    }

    min_cost
}

fn main() {
    input! {
        n: usize,
        x: i64,
        teeth: [(i64, i64); n],
    }

    let result = solve(n, x, teeth);
    println!("{}", result);
}
