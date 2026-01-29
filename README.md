# Cosmic Calculator

An unofficial calculator application for the Cosmic Desktop Environment.

## Features

- Basic arithmetic operations (addition, subtraction, multiplication, division)
- Clean and intuitive user interface following Cosmic DE design principles
- Decimal number support
- Backspace to correct input mistakes
- Error handling for division by zero

## Building

### Prerequisites

- Rust toolchain (1.70 or later)
- libcosmic dependencies

### Build from source

```bash
cargo build --release
```

The compiled binary will be located at `target/release/cosmic-calculator`.

## Installation

### From source

```bash
cargo install --path .
```

### Desktop Entry

To add the application to your system menu, copy the desktop file:

```bash
sudo cp res/com.github.jepomeroy.CosmicCalculator.desktop /usr/share/applications/
```

## Usage

Run the calculator:

```bash
cosmic-calculator
```

## License

This project is licensed under the MIT License.