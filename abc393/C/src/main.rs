use proconio::input;
use std::collections::HashMap;

fn main() {
    input! {
        n: usize,  // 頂点数
        m: usize,  // 辺数
        edges: [(usize, usize); m],  // 辺のリスト
    }

    println!("{}", solve(m, &edges));
}

// nは使用しないので削除
fn solve(m: usize, edges: &[(usize, usize)]) -> usize {
    if m == 0 {
        return 0;
    }

    let mut remove_count = 0;
    let mut edge_counts: HashMap<(usize, usize), usize> = HashMap::new();

    // 各辺を処理
    for &(mut u, mut v) in edges {
        // 自己ループの検出
        if u == v {
            remove_count += 1;
            continue;
        }

        // 辺を正規化（小さい頂点番号を先に）
        if u > v {
            std::mem::swap(&mut u, &mut v);
        }

        // 多重辺の検出
        let count = edge_counts.entry((u, v)).or_insert(0);
        *count += 1;
    }

    // 多重辺の数をカウント（修正部分）
    for &count in edge_counts.values() {
        if count > 1 {
            // count - 1 は正しい：各辺のグループで1本だけ残して他を削除
            remove_count += count - 1;
        }
    }

    remove_count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example_1() {
        let edges = vec![(1, 2), (2, 3), (3, 2), (3, 1), (1, 1)];
        assert_eq!(solve(5, &edges), 2);
    }

    #[test]
    fn test_example_2() {
        let edges = vec![];
        assert_eq!(solve(0, &edges), 0);
    }

    #[test]
    fn test_example_3() {
        let edges = vec![
            (6, 2),
            (4, 1),
            (5, 1),
            (6, 6),
            (5, 3),
            (5, 1),
            (1, 4),
            (6, 4),
            (4, 2),
            (5, 6),
        ];
        assert_eq!(solve(10, &edges), 3);
    }

    #[test]
    fn test_only_self_loops() {
        let edges = vec![(1, 1), (2, 2), (1, 1)];
        assert_eq!(solve(3, &edges), 3);
    }

    #[test]
    fn test_only_multiple_edges() {
        let edges = vec![(1, 2), (1, 2), (2, 1), (2, 1)];
        assert_eq!(solve(4, &edges), 2);
    }

    #[test]
    fn test_no_redundant_edges() {
        let edges = vec![(1, 2), (2, 3), (3, 4)];
        assert_eq!(solve(3, &edges), 0);
    }

    #[test]
    fn test_complex_case() {
        let edges = vec![
            (1, 1),
            (2, 2),
            (1, 2),
            (1, 2),
            (3, 4),
            (3, 4),
            (4, 5),
            (5, 3),
        ];
        assert_eq!(solve(8, &edges), 4);
    }
}
