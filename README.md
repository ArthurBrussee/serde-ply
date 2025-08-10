# serde_ply

Flexible and fast PLY parser and writer using serde. While PLY is an older format, it's still used in various geometry processing applications and research. PLY files act as simple key-value tables, and can be decoded in a streaming manner.

## Features

- Supports serializing and deserializing PLY files
- Supports full Ply specification including list properties
- Supports binary and ascii formats
- Supports deserializing PLY files in chunks, for streaming data processing.
- High performance (1 GB/s+ deserialization)

## Examples

Please see the examples folder for a few basic usages of the library.

## Contributions

Contributions are welcome! Please open an issue or submit a pull request. Please run the tests to check if everything still works, and `cargo bench` to check if performance is still acceptable.
