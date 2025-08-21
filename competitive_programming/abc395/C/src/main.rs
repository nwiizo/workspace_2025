use proconio::input;
use std::collections::HashMap;

fn main() {
    input! {
        n: usize,
        a: [i32; n],
    }

    // 各値の最後の出現位置を記録するHashMap
    let mut last_position = HashMap::new();

    // 最短の長さを保持する変数（初期値は無限大）
    let mut min_length = usize::MAX;

    // 配列を順に走査
    for (current_pos, &value) in a.iter().enumerate() {
        // この値が以前に出現したか確認
        if let Some(&prev_pos) = last_position.get(&value) {
            // 同じ値が見つかった場合、その間の連続部分列の長さを計算
            // (現在の位置 - 前回の位置 + 1)
            let length = current_pos - prev_pos + 1;
            // 最短の長さを更新
            min_length = min_length.min(length);
        }

        // 現在の値と位置を記録または更新
        last_position.insert(value, current_pos);
    }

    // 結果を出力
    if min_length == usize::MAX {
        // 条件を満たす連続部分列が見つからなかった場合
        println!("-1");
    } else {
        // 見つかった場合はその最短の長さを出力
        println!("{}", min_length);
    }
}
