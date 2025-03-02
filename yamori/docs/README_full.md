# Yamori

Yamori is a test runner and visualizer for command-line applications. It allows you to define tests in TOML or YAML format and visualize the results in a terminal UI or run them in CLI mode.

## Features

- Define tests in TOML or YAML format
- Run commands with arguments and input
- Compare actual output with expected output
- Visualize test results in a terminal UI or simple CLI output
- Support for timeouts
- Support for pre-build commands
- Per-test build configuration
- Color-coded test results

## Directory Structure

```
yamori/
├── src/                 # Source code
├── examples/            # Example applications
└── tests/
    └── configs/         # Test configuration files
        ├── tests.toml   # TOML configuration
        └── tests.yaml   # YAML configuration
```

## Usage

### TUI Mode (Default)

You can run Yamori with a specific configuration file using one of the following methods:

1. Using the `-y` or `--yamori-config` flag:
   ```
   cargo run -- --yamori-config tests/configs/tests.yaml
   ```

2. Using the `YAMORI_CONFIG` environment variable:
   ```
   YAMORI_CONFIG=tests/configs/tests.yaml cargo run
   ```

The environment variable takes precedence over the command-line flag if both are specified.

### CLI Mode

You can run Yamori in CLI mode (without the TUI interface) using the `-c` or `--cli` flag:

```
cargo run -- --cli --yamori-config tests/configs/tests.yaml
```

Or with the environment variable:

```
YAMORI_CONFIG=tests/configs/tests.yaml cargo run -- --cli
```

In CLI mode, Yamori will run all tests and display a compact summary of the results. Only failed tests will show detailed information. This is useful for CI/CD pipelines or when you want a quick overview of test results.

## Configuration Format

Yamori supports both TOML and YAML configuration files. The file format is automatically detected based on the file extension (`.toml`, `.yaml`, or `.yml`).

### Build Configuration

Yamori supports both global and per-test build configurations:

- **Global build configuration**: Defined at the root level of the configuration file. Used as a fallback for tests that don't specify their own build settings.
- **Per-test build configuration**: Defined within each test. Takes precedence over the global configuration.

## Example Configuration (TOML)

```toml
# Global build configuration
[build]
release = false
pre_build_commands = ["echo 'Global build preparation'"]

# Test definitions
[[tests]]
name = "Echo Test"
command = "echo"
args = ["Hello, World!"]
expected_output = "Hello, World!"
timeout_secs = 5
[tests.build]
release = false
pre_build_commands = ["echo 'Preparing Echo Test'"]

[[tests]]
name = "Word Count Test"
command = "wc"
args = ["-w"]
input = "This is a test sentence with exactly eight words."
expected_output = "8"
timeout_secs = 5
```

## Key Bindings (TUI Mode)

In the terminal UI:

- `q`: Quit
- `?`: Toggle help
- `j` or Down Arrow: Move down
- `k` or Up Arrow: Move up
- `h` or Left Arrow: Previous tab
- `l` or Right Arrow: Next tab
- `r`: Re-run tests
- `b`: Toggle release mode
- `R`: Run tests in release mode
- `Esc`: Close help

## License

[MIT License](LICENSE) 