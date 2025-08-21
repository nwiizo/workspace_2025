# Yew Pomodoro Timer

A Pomodoro timer application built with Rust and Yew framework, compiled to WebAssembly. This application helps you manage your work sessions using the Pomodoro Technique, allowing you to track tasks and their completion times.

## Features

- 25-minute countdown timer with start, stop, and reset controls
- Task management system for tracking current and completed tasks
- Task history with actual work duration tracking
- Export completed tasks in Markdown format
- Clipboard integration for easy task history export
- Modern UI with Tailwind CSS styling

## Prerequisites

Before you begin, ensure you have installed:

- Rust (v1.75.0 or later)
- `wasm32-unknown-unknown` target
- Trunk (for building and serving the application)

## Setup

1. Install the required Rust target:
```bash
rustup target add wasm32-unknown-unknown
```

2. Install Trunk:
```bash
cargo install trunk
```


## Running the Application

To run the application in development mode:

```bash
trunk serve
```

This will start a development server at `http://127.0.0.1:8080` (by default).

## Project Structure

- `src/main.rs`: Main application code including the timer implementation
- `Cargo.toml`: Project dependencies and configuration
- `index.html`: HTML template for the application

## Dependencies

- `yew`: Frontend framework for Rust
- `gloo-timers`: Timer functionality
- `web-sys`: Web APIs bindings
- `chrono`: Date and time functionality
- `wasm-bindgen`: WebAssembly bindings

## Features Implementation

### Timer
- 25-minute countdown timer
- Visual feedback for timer status
- Controls for starting, stopping, and resetting

### Task Management
- Input field for current task description
- Task completion tracking with timestamps
- History of completed tasks with duration

### Data Export
- Markdown formatting for task history
- Clipboard integration for easy sharing
- Modal display for exported content

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- Built with [Yew](https://yew.rs/)
- Inspired by the Pomodoro Technique
- UI styled with Tailwind CSS
