# Yamori

Yamori is a test runner and visualizer for command-line applications. It allows you to define tests in TOML or YAML format and visualize the results in a terminal UI.

## Features

- Define tests in TOML or YAML format
- Run commands with arguments and input
- Compare actual output with expected output
- Visualize test results in a terminal UI
- Support for timeouts
- Support for pre-build commands
- Per-test build configuration

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

## Configuration Format

Yamori supports both TOML and YAML configuration files. The file format is automatically detected based on the file extension (`.toml`, `.yaml`, or `.yml`).

### Build Configuration

Yamori supports both global and per-test build configurations:

- **Global build configuration**: Defined at the root level of the configuration file. Used as a fallback for tests that don't specify their own build settings.
- **Per-test build configuration**: Defined within each test. Takes precedence over the global configuration.

## Key Bindings

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