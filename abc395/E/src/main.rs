use proconio::input;
use std::cmp::Reverse;
use std::collections::BinaryHeap;

fn main() {
    input! {
        n: usize,
        m: usize,
        x: u64,
        edges: [(usize, usize); m],
    }

    // 順方向と逆方向の隣接リストを作成
    let mut forward = vec![vec![]; n + 1];
    let mut reverse = vec![vec![]; n + 1];

    for &(u, v) in &edges {
        forward[u].push(v);
        reverse[v].push(u);
    }

    // Dijkstraで最短経路を計算
    let mut dist = vec![vec![u64::MAX; 2]; n + 1]; // [頂点][反転フラグ]
    let mut heap = BinaryHeap::new();

    // 初期状態：頂点1、反転なし
    dist[1][0] = 0;
    heap.push((Reverse(0), 1, 0)); // (コスト, 頂点, 反転フラグ)

    while let Some((Reverse(cost), v, flipped)) = heap.pop() {
        // 既に処理済みの状態ならスキップ
        if dist[v][flipped] < cost {
            continue;
        }

        // グラフの選択
        let graph = if flipped == 0 { &forward } else { &reverse };

        // 通常の移動（コスト1）
        for &next_v in &graph[v] {
            let next_cost = cost + 1;
            if dist[next_v][flipped] > next_cost {
                dist[next_v][flipped] = next_cost;
                heap.push((Reverse(next_cost), next_v, flipped));
            }
        }

        // 反転操作を考慮（反転フラグを切り替え）
        let next_flipped = 1 - flipped;
        let next_cost = cost + x;
        if dist[v][next_flipped] > next_cost {
            dist[v][next_flipped] = next_cost;
            heap.push((Reverse(next_cost), v, next_flipped));
        }
    }

    // 頂点Nに到達する最小コスト
    let min_cost = dist[n][0].min(dist[n][1]);

    println!("{}", min_cost);
}
