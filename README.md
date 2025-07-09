# render_sandbox

A sandbox for trying out different graphics rendering algorithms and learning Rust.

## Installation

Make sure you have Rust installed, then clone and build the project:

```bash
git clone https://github.com/mpiispanen/render_sandbox.git
cd render_sandbox
cargo build --release
```

## Usage

```bash
# Run with default settings (800x600 resolution, output.png)
./target/release/render_sandbox

# Specify custom resolution and output file
./target/release/render_sandbox --width 1920 --height 1080 --output render.jpg --format jpg

# Use short options with verbose output
./target/release/render_sandbox -w 1024 -f png -s 4 -v

# View all available options
./target/release/render_sandbox --help
```

### Command Line Options

- `-w, --width <WIDTH>`: Rendering resolution width (default: 800)
- `--height <HEIGHT>`: Rendering resolution height (default: 600)
- `-o, --output <OUTPUT>`: Output file path (default: output.png)
- `-f, --format <FORMAT>`: Output format - png, jpg, jpeg, or bmp (default: png)
- `-s, --samples <SAMPLES>`: Number of samples for anti-aliasing (default: 1)
- `-v, --verbose`: Enable verbose output
- `-h, --help`: Show help information
- `-V, --version`: Show version information

## Development

### Building

```bash
cargo build
```

### Running Tests

```bash
cargo test
```

### Code Formatting

```bash
cargo fmt
```

### Linting

```bash
cargo clippy
```

## CI/CD

This project uses GitHub Actions for continuous integration. The CI pipeline runs on every push and pull request, executing:

- Code formatting checks
- Clippy linting
- Unit tests
- Cross-platform builds (Linux, macOS, Windows)

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.