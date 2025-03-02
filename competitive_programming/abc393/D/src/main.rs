use proconio::input;

fn main() {
    input! {
        n: usize,
        s: String,
    }
    println!("{}", solve(n, &s));
}

fn solve(n: usize, s: &str) -> usize {
    // 1のインデックスを全て取得
    let one_positions: Vec<usize> = s
        .chars()
        .enumerate()
        .filter(|&(_, c)| c == '1')
        .map(|(i, _)| i)
        .collect();

    // 1がすでに連続している場合は0を返す
    if one_positions.windows(2).all(|w| w[1] - w[0] == 1) {
        return 0;
    }

    // 1を連続させるために必要な移動回数を計算
    let target_start = one_positions.len() / 2;
    let target_pos = one_positions[target_start];

    let mut moves = 0;
    for (i, &pos) in one_positions.iter().enumerate() {
        let target = target_pos - target_start + i;
        moves += pos.abs_diff(target);
    }

    moves
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example_1() {
        assert_eq!(solve(7, "0101001"), 3);
    }

    #[test]
    fn test_example_2() {
        assert_eq!(solve(3, "100"), 0);
    }

    #[test]
    fn test_example_3() {
        assert_eq!(solve(10, "0101001001"), 7);
    }

    #[test]
    fn test_already_grouped() {
        assert_eq!(solve(5, "11100"), 0);
        assert_eq!(solve(5, "00111"), 0);
    }

    #[test]
    fn test_single_one() {
        assert_eq!(solve(5, "10000"), 0);
        assert_eq!(solve(5, "00010"), 0);
    }

    #[test]
    fn test_alternating() {
        assert_eq!(solve(6, "101010"), 3);
    }

    #[test]
    fn test_complex_cases() {
        assert_eq!(solve(8, "10010011"), 2);
        assert_eq!(solve(10, "1001001001"), 6);
    }

    #[test]
    fn test_edge_cases() {
        // 最小ケース（N=2）
        assert_eq!(solve(2, "10"), 0);
        assert_eq!(solve(2, "01"), 1);

        // 両端に1がある場合
        assert_eq!(solve(5, "10001"), 2);
    }

    #[test]
    fn test_scattered_ones() {
        assert_eq!(solve(10, "1000100010"), 4);
    }
}
