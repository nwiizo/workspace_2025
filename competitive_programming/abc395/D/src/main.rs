use proconio::input;
use std::io::{self, Write};

fn main() {
    // 高速化のためのバッファリングされた出力
    let out = io::stdout();
    let mut out = io::BufWriter::with_capacity(1024 * 1024, out.lock());

    input! {
        n: usize,
        q: usize,
    }

    // 巣の「機能」を追跡
    // nest_function[i] = 巣iの「機能」の番号
    let mut nest_function = vec![0; n + 1];
    for i in 1..=n {
        nest_function[i] = i;
    }

    // 機能から巣への逆マッピング
    // function_to_nest[f] = 機能fを持つ巣の番号
    let mut function_to_nest = vec![0; n + 1];
    for i in 1..=n {
        function_to_nest[i] = i;
    }

    // 鳩がいる巣の「機能」を追跡
    // pigeon_function[i] = 鳩iがいる巣の「機能」の番号
    let mut pigeon_function = vec![0; n + 1];
    for i in 1..=n {
        pigeon_function[i] = i;
    }

    // クエリ処理
    for _ in 0..q {
        input! {
            op_type: u8,
        }

        match op_type {
            1 => {
                // 種類1: 鳩aを巣bに移動
                input! {
                    a: usize,
                    b: usize,
                }

                // 鳩aを巣bの「機能」に対応する巣に移動
                pigeon_function[a] = nest_function[b];
            }
            2 => {
                // 種類2: 巣aと巣bの鳩を交換
                input! {
                    a: usize,
                    b: usize,
                }

                // 巣aと巣bの「機能」を交換
                let function_a = nest_function[a];
                let function_b = nest_function[b];

                nest_function[a] = function_b;
                nest_function[b] = function_a;

                // 逆マッピングも更新
                function_to_nest[function_a] = b;
                function_to_nest[function_b] = a;
            }
            3 => {
                // 種類3: 鳩aがいる巣の番号を報告
                input! {
                    a: usize,
                }

                // 鳩aがいる巣の「機能」から、対応する巣の番号を取得
                let nest_number = function_to_nest[pigeon_function[a]];

                writeln!(out, "{}", nest_number).unwrap();
            }
            _ => unreachable!(),
        }
    }
}
