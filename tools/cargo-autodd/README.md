# Cargo AutoAdd

Cargo AutoAdd is a tool that automatically manages dependencies in Rust projects. It analyzes your source code to detect used crates and updates `Cargo.toml` accordingly.

## Features

- Automatically scans `.rs` files in your project
- Detects dependencies from `use` statements and `extern crate` declarations
- Automatically fetches the latest stable versions from crates.io
- Detects and removes unused dependencies (while protecting essential ones)
- Optional rust-analyzer integration for better analysis

## Installation

```bash
git clone https://github.com/nwiizo/cargo-autodd
cd cargo-autodd
cargo install --path .
```

## Requirements

- Rust 1.70.0 or later
- Cargo
- rust-analyzer (optional, recommended for better analysis)

## Usage

Run the following command in your project's root directory:

```bash
cargo run
```

## How It Works

1. Scans all `.rs` files in your project
2. Analyzes `use` statements and `extern crate` declarations
3. Performs more accurate analysis if rust-analyzer is available
4. Fetches latest versions of detected crates from crates.io
5. Updates your `Cargo.toml`
6. Runs `cargo check` to verify changes

## Limitations

- May not detect dependencies introduced by macros
- Limited support for conditional compilation (`cfg` attributes)
- Limited workspace support
- Does not handle complex feature flag dependencies

## Contributing

Pull requests are welcome! Here's how you can contribute:

1. Fork the repository
2. Create a feature branch (`git checkout -b my-new-feature`)
3. Commit your changes (`git commit -am 'Add some feature'`)
4. Push to the branch (`git push origin my-new-feature`)
5. Create a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Author

- nwiizo (@nwiizo)

## Acknowledgments

This project uses the following tools and libraries:

- [rust-analyzer](https://rust-analyzer.github.io/)
- [toml_edit](https://docs.rs/toml_edit/)
- [walkdir](https://docs.rs/walkdir/)
- [semver](https://docs.rs/semver/)

## Development Status

This project is currently in active development. Features and APIs may change.
