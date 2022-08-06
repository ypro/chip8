# Emulator of CHIP-8 machine in Rust.

My attempt to write a simple emulator of CHIP-8 machine in Rust.

# Build
```
cargo build --release
```
 
# Usage
```
cargo run --release -- [OPTIONS]

OPTIONS:
    -f, --fast                 Run emulation as fast as possible.
    -h, --help                 Print help information
    -p, --profile <profile>    Chip-8 profile. [default: modern] [possible values: original, modern]
    -r, --rom_path <path>      ROM file name. [default: rom/tests/ibm.ch8]
    -V, --version              Print version information
```

# Run test cases
```
cargo test
```
