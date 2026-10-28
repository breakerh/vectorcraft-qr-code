# VectorCraft QR Code Plugin

Turn text or a URL into an editable vector QR code in your open VectorCraft document. Works entirely offline. Tested with VectorCraft 0.17.0.

**[Download the plugin](https://github.com/breakerh/vectorcraft-qr-code/releases/latest/download/vectorcraft-qr-code.wasm)** · [Releases](https://github.com/breakerh/vectorcraft-qr-code/releases)

## Installation

1. Download `vectorcraft-qr-code.wasm` from the link above.
2. In VectorCraft, choose **Object → Plug-ins → Install Plug-in…** and select the file. You can also use **File → Open**.
3. Accept the prompt for a larger computation budget. This allows longer input to be encoded; the plugin runs locally in the background and can be cancelled.

Use the installation command rather than the Additional Plug-ins Folder: folder loading skips plugins that request a larger computation budget. No Rust installation or compilation is needed to use the downloaded plugin.

## Usage

1. Open your document and preferably deselect any existing artwork.
2. Choose **Object → Plug-ins → Generate QR Code…**.
3. Enter your text or a complete URL, such as `https://example.com`.
4. Set the size in millimeters, including the quiet zone, and choose an error correction level. M is a good default.
5. Leave **White background** enabled to keep the quiet zone light. Disable it only when the underlying artwork already provides a plain white background.
6. Click **OK**. The plugin adds the code at the center of the artboard as a black vector path containing closed subpaths, plus an optional separate white background.

Group, move, recolor, or scale the resulting artwork proportionally. One Undo reverses the insertion. Existing selected objects are preserved, but VectorCraft may keep them selected together with the new QR code.

Codes are static. Running the plugin again creates a new code; to change a URL, generate a new code and remove the previous one.

## Features and limits

- Editable vector paths; no embedded raster image.
- Entirely offline: no network access, account, subscription, or tracking service.
- Sizes from 10 to 1000 mm, including a quiet zone of four QR modules on each side.
- L, M, Q, and H error correction. The encoder may increase correction if it fits within the same QR dimensions.
- Unicode text encoded as UTF-8; leading and trailing spaces are preserved.
- Automatic QR version selection. Input that exceeds QR capacity produces an error; capacity depends on content and error correction. The text field accepts up to 4096 characters.
- Dense codes at small print sizes have very small modules. Test the final printed result with a scanner.

The plugin uses VectorCraft plug-in ABI v1 with string parameters and the heavy computation budget. Older hosts that lack these features may not support it.

## Build from source

Install Rust with the `wasm32-unknown-unknown` target, then run:

```sh
rustup target add wasm32-unknown-unknown
cargo test --locked
cargo build --locked --release --target wasm32-unknown-unknown
```

The compiled plugin is `target/wasm32-unknown-unknown/release/vectorcraft_qr_code.wasm`. Rename it to `vectorcraft-qr-code.wasm` for distribution.

## Validation

- Three Rust tests cover preservation of existing objects, dimensions and placement, invalid input, Unicode, and a transparent background.
- 24 runs of the compiled WASM were decoded with ZXing and matched their input byte for byte. Cases include URL query parameters, Unicode, spaces, and 1000 bytes of text at all four error correction levels.
- Tested with wasmi 2.0.0 using the host computation budget, up to 2953 bytes at level L. Inputs beyond QR capacity return an error.
- The WASM has no host imports.
- Native VectorCraft installation, parameter dialog, and vector preview were checked with the default URL before the English translation. A full native UI retest of the English build remains outstanding.

The checked-in JSON reports record the encoder validation. Translation does not change the QR encoding algorithm.

## License and references

MIT license. Dependency notices are included in [THIRD-PARTY-NOTICES.txt](THIRD-PARTY-NOTICES.txt).

- [VectorCraft plug-in API](https://github.com/storytold/vectorcraft/blob/main/docs/plugins.md)
- [Nayuki QR Code generator](https://github.com/nayuki/QR-Code-generator)
