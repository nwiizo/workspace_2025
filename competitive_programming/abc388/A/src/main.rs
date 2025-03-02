use proconio::input;

fn main() {
    use proconio::input;

    // Read input from the user
    input! {
        input: String,
    }

    // Trim the input to remove any trailing newline characters
    let input = input.trim();

    // Extract the first character
    let first_char = input.chars().next().unwrap();

    // Concatenate the first character with "UPC"
    let result = format!("{}UPC", first_char);

    // Print the result
    println!("{}", result);
}
