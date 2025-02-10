use proconio::input;

fn main() {
    input! {
        n: usize,
        d: usize,
        snakes: [(i32, i32); n],  // (thickness, length) pairs
    }

    // For each day k (1-based)
    for k in 1..=d {
        let mut max_weight = 0;

        // Calculate weight for each snake
        for &(thickness, initial_length) in snakes.iter() {
            // New length after k days
            let new_length = initial_length + k as i32;
            // Calculate weight (thickness * length)
            let weight = thickness * new_length;
            // Update max weight if current snake is heavier
            max_weight = max_weight.max(weight);
        }

        println!("{}", max_weight);
    }
}

#[cfg(test)]
mod tests {
    use proconio::input;
    use proconio::source::auto::AutoSource;

    fn solve(input: &str) -> Vec<i32> {
        let source = AutoSource::from(input);
        input! {
            from source,
            n: usize,
            d: usize,
            snakes: [(i32, i32); n],
        }

        let mut result = Vec::new();
        for k in 1..=d {
            let mut max_weight = 0;
            for &(thickness, initial_length) in snakes.iter() {
                let new_length = initial_length + k as i32;
                let weight = thickness * new_length;
                max_weight = max_weight.max(weight);
            }
            result.push(max_weight);
        }
        result
    }

    #[test]
    fn test_example1() {
        let input = "4 3\n3 3\n5 1\n2 4\n1 10\n";
        let expected = vec![12, 15, 20];
        assert_eq!(solve(input), expected);
    }

    #[test]
    fn test_example2() {
        let input = "1 4\n100 100\n";
        let expected = vec![10100, 10200, 10300, 10400];
        assert_eq!(solve(input), expected);
    }

    #[test]
    fn test_single_snake() {
        let input = "1 1\n1 1\n";
        let expected = vec![2];
        assert_eq!(solve(input), expected);
    }

    #[test]
    fn test_same_weights() {
        let input = "3 2\n2 2\n2 2\n2 2\n";
        let expected = vec![6, 8];
        assert_eq!(solve(input), expected);
    }

    #[test]
    fn test_max_constraints() {
        let input = "2 100\n100 100\n1 1\n";
        let mut expected = Vec::new();
        for k in 1..=100 {
            expected.push(100 * (100 + k));
        }
        assert_eq!(solve(input), expected);
    }

    #[test]
    fn test_minimal_values() {
        let input = "2 3\n1 1\n1 1\n";
        let expected = vec![2, 3, 4];
        assert_eq!(solve(input), expected);
    }
}
