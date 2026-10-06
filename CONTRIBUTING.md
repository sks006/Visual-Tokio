# Contributing to Tokio Visual Guide

Thank you for your interest in improving the Tokio Visual Guide! This project combines interactive browser-based visual animations with production-grade Rust asynchronous examples to explain Tokio's core architecture and APIs.

## Project Structure

- `index.html`: The main interactive dashboard, hosting all simulations, architectural maps, and the API reference.
- `*.html`: Standalone interactive simulations (`select.html`, `async_io.html`, `tracing.html`, `accept.html`, `echo.html`, `mpsc.html`).
- `examples/`: Runnable Rust examples demonstrating each Tokio concept in actual code.
- `asset/`: High-resolution architectural infographics and flowcharts.
- `readme.md`: Master reference document with embedded Mermaid diagrams and deep architectural explanations.

## Development Workflow

### Web / Visualizers
1. Open `index.html` directly in any modern browser, or run a local dev server:
   ```bash
   npx serve .
   # or
   python3 -m http.server 8080
   ```
2. Ensure animations are smooth, accessible, cancel-safe, and visually consistent with the dark theme.

### Rust Code
1. Verify all examples compile and run:
   ```bash
   cargo check --examples
   cargo test
   ```
2. Run any specific example:
   ```bash
   cargo run --example 01_select_timeout
   cargo run --example 02_async_io_reactor
   cargo run --example 03_tracing_console
   cargo run --example 04_tcp_accept_resilient
   cargo run --example 05_echo_server
   cargo run --example 06_mpsc_backpressure
   ```
3. Check formatting and lints:
   ```bash
   cargo fmt --all -- --check
   cargo clippy --all-targets -- -D warnings
   ```

## Pull Request Guidelines

1. Make sure all Rust code compiles without warnings and all examples exit gracefully.
2. If adding a new animation, add a corresponding Rust example in `examples/` and register it in `index.html`.
3. Keep animations lightweight (vanilla HTML/CSS/JS without heavy external framework bloat).
