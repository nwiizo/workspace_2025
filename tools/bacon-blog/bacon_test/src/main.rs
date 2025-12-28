/// 簡単な計算関数
///
/// # Examples
///
/// ```
/// let result = add_numbers(2, 3);
/// assert_eq!(result, 5);
/// ```
pub fn add_numbers(a: i32, b: i32) -> i32 {
    a + b
}

/// 未使用の変数を含む関数（Clippy警告テスト用）
pub fn clippy_test_function() {
    let unused_variable = 42;
    println!("Testing clippy warnings");
}

/// 型エラーをテストする関数（コメントアウト状態）
// pub fn type_error_test() {
//     let x: i32 = "this is a string"; // 型エラー
// }

fn main() {
    println!("Bacon test project!");
    let result = add_numbers(10, 20);
    println!("Result: {}", result);
    clippy_test_function();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_numbers() {
        assert_eq!(add_numbers(2, 3), 5);
        assert_eq!(add_numbers(-1, 1), 0);
    }

    #[test]
    fn test_add_negative() {
        assert_eq!(add_numbers(-5, -3), -8);
    }
}
