use proconio::input;

fn main() {
    input! {
        n: usize,
        mut stones: [i32; n],
    }

    let mut active_givers: i32 = 0; // 現在石を持っている成人の数
    let mut lose_stones = vec![0; n]; // i年目に石を失う人の数

    // 各年のシミュレーション
    for i in 0..n {
        // 石を失う人を反映
        if i > 0 {
            active_givers = active_givers.saturating_sub(lose_stones[i - 1]);
        }

        // 現在の成人者から石を受け取る
        stones[i] += active_givers;

        // この人が石を持っているなら、future_giversに追加
        if stones[i] > 0 {
            let stones_can_give = stones[i].min((n - i - 1) as i32);
            if stones_can_give > 0 {
                active_givers += 1;
                lose_stones[i] += 1;
                if i + (stones_can_give as usize) < n {
                    lose_stones[i + (stones_can_give as usize)] += 1;
                }
                stones[i] -= stones_can_give;
            }
        }
    }

    // 結果の出力
    println!(
        "{}",
        stones
            .iter()
            .map(|&x| x.to_string())
            .collect::<Vec<_>>()
            .join(" ")
    );
}
