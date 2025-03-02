use proconio::input;
use std::collections::HashMap;

fn solve(n: usize, k: usize, a: &[usize]) -> Vec<usize> {
    // 配列 a の最大値を取得
    let max_a = *a.iter().max().unwrap();

    // 各値の出現頻度を数える（ただし、同じ値が複数あっても、must_includeは1回として扱うための準備）
    let mut freq = vec![0; max_a + 1];
    for &x in a {
        freq[x] += 1;
    }

    // 各整数 d について、a の中で「d で割り切れる数の個数」を求める
    // ここではエラトステネス的手法で、d の倍数を走査します
    let mut count = vec![0; max_a + 1];
    for d in 1..=max_a {
        let mut m = d;
        while m <= max_a {
            count[d] += freq[m];
            m += d;
        }
    }

    // 同じ値に対しては、約数の列挙結果をキャッシュして重複計算を避ける
    let mut cache: HashMap<usize, usize> = HashMap::new();
    let mut result = Vec::with_capacity(n);

    // 各対象値 v（must_include）について、v の約数のうち、
    // 「(global count) - (vの余分な出現数)」が k 以上となる最大の約数を求める
    // ※ここで effective_count = count[d] - freq[v] + 1 としているのは、vが複数回現れる場合に
    //    v 自体は1回だけカウントするためです。
    for &v in a {
        if let Some(&ans) = cache.get(&v) {
            result.push(ans);
            continue;
        }
        let mut divisors = Vec::new();
        let r = (v as f64).sqrt() as usize;
        for i in 1..=r {
            if v % i == 0 {
                divisors.push(i);
                if i != v / i {
                    divisors.push(v / i);
                }
            }
        }
        // 降順に調べる
        divisors.sort_unstable_by(|a, b| b.cmp(a));

        let mut best = 1;
        for d in divisors {
            // effective_count: 全体で d で割り切れる数 count[d] から、
            // v の出現回数を全て引いて（もし複数あっても v は1回とする）、
            // 1 を足す
            if count[d] - freq[v] + 1 >= k {
                best = d;
                break;
            }
        }
        cache.insert(v, best);
        result.push(best);
    }
    result
}

fn main() {
    input! {
        n: usize,
        k: usize,
        a: [usize; n]
    }
    let res = solve(n, k, &a);
    for x in res {
        println!("{}", x);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example_1() {
        let n = 5;
        let k = 2;
        let a = vec![3, 4, 6, 7, 12];
        let expected = vec![3, 4, 6, 1, 6];
        assert_eq!(solve(n, k, &a), expected);
    }

    #[test]
    fn test_example_2() {
        let n = 3;
        let k = 3;
        let a = vec![6, 10, 15];
        let expected = vec![1, 1, 1];
        assert_eq!(solve(n, k, &a), expected);
    }

    #[test]
    fn test_example_3() {
        let a = vec![
            414003, 854320, 485570, 52740, 833292, 625990, 909680, 885153, 435420, 221663,
        ];
        let expected = vec![59, 590, 590, 879, 879, 590, 20, 879, 590, 59];
        assert_eq!(solve(10, 3, &a), expected);
    }
}
