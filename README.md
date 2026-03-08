# μTools

A small utility suite for developers, built with Rust and [GPUI](https://gpui.rs).

![μTools](assets/icons/utools.svg)

## Tools

### Base64

- **Base64 Encoder** — Converts text into a Base64 encoded string.
- **Base64 Decoder** — Converts a Base64 encoded string into text.

### Data URL

- **Data URL Generator** — Converts text into a data URL.

### Date/Time

- **Unix Timestamp Converter** — Transforms Unix timestamps into human-readable date and time formats.

### HTML

- **HTML Encoder** — Converts text into an HTML encoded string.
- **HTML Decoder** — Converts an HTML encoded string into text.

### JSON

- **JSON Formatter** — Formats or compacts JSON data for better structure and clarity.
- **JSON Viewer** — Interactively browse and inspect JSON data.

### Text

- **Text Character Count** — Counts characters in any text and displays the total.
- **Text Difference** — Shows differences between two texts.

## Building

Requires Rust (2024 edition), Clang, and `tree-sitter`.

```sh
cargo build --release
```

The binary will be at `target/release/utools`.

You can also open a specific tool directly from the command line:

```sh
utools "Base64 Encoder"
```

## Installing (Arch Linux)

```sh
makepkg -si
```

## License

MIT
