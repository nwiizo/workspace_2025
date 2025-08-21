# Test Configurations

This directory contains configuration files for Yamori tests.

## File Formats

Yamori supports both TOML and YAML configuration files:

- `tests.toml`: Test configuration in TOML format
- `tests.yaml`: Test configuration in YAML format

## Usage

You can run Yamori with a specific configuration file using one of the following methods:

1. Using the `-y` or `--yamori-config` flag:
   ```
   # For TOML configuration
   cargo run -- --yamori-config tests/configs/tests.toml
   
   # For YAML configuration
   cargo run -- --yamori-config tests/configs/tests.yaml
   ```

2. Using the `YAMORI_CONFIG` environment variable:
   ```
   # For TOML configuration
   YAMORI_CONFIG=tests/configs/tests.toml cargo run
   
   # For YAML configuration
   YAMORI_CONFIG=tests/configs/tests.yaml cargo run
   ```

The environment variable takes precedence over the command-line flag if both are specified.

## Configuration Format

Yamori supports both TOML and YAML configuration files. The file format is automatically detected based on the file extension (`.toml`, `.yaml`, or `.yml`).

### TOML Example

```toml
# Test configuration in TOML format
[[tests]]
name = "Echo Test"
command = "echo"
args = ["Hello, World!"]
expected_output = "Hello, World!"
timeout_secs = 5
# Per-test build configuration
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
# Per-test build configuration
[tests.build]
release = true
pre_build_commands = ["echo 'Preparing Word Count Test'", "echo 'Running in release mode'"]

# Global build configuration (used as fallback if test doesn't specify its own)
[build]
release = false
pre_build_commands = ["echo 'Global build preparation'"]
```

### YAML Example

```yaml
# Test configuration in YAML format
tests:
  - name: "Echo Test"
    command: "echo"
    args: ["Hello, World!"]
    expected_output: "Hello, World!"
    timeout_secs: 5
    # Per-test build configuration
    build:
      release: false
      pre_build_commands:
        - "echo 'Preparing Echo Test'"

  - name: "Word Count Test"
    command: "wc"
    args: ["-w"]
    input: "This is a test sentence with exactly eight words."
    expected_output: "8"
    timeout_secs: 5
    # Per-test build configuration
    build:
      release: true
      pre_build_commands:
        - "echo 'Preparing Word Count Test'"
        - "echo 'Running in release mode'"

# Global build configuration (used as fallback if test doesn't specify its own)
build:
  release: false
  pre_build_commands:
    - "echo 'Global build preparation'"
```

## Build Configuration

Yamori supports both global and per-test build configurations:

- **Global build configuration**: Defined at the root level of the configuration file. Used as a fallback for tests that don't specify their own build settings.
- **Per-test build configuration**: Defined within each test. Takes precedence over the global configuration.

Each build configuration can include:

- `release`: Boolean flag indicating whether to build in release mode
- `pre_build_commands`: List of commands to run before executing the test 