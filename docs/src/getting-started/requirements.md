# Requirements

## Operating Systems

Sauva supports macOS and Linux. Windows is not currently supported.

## Terminal

An interactive terminal is required, including when the input is read from a pipe or file. Sauva reads the supplied input first, then accepts keyboard input from the terminal.

The minimum terminal size is 60 columns by 16 rows. A larger terminal, such as 100 columns by 30 rows, provides room for additional preview panels.

## Glyph Images

Glyph images require a compatible terminal and fonts installed on your system. Sauva does not bundle fonts.

kitty and Ghostty are supported for automatic graphics detection. See [Compatibility](./compatibility.md) for the available protocols and modes.

Unicode properties, browsing, searching, and text analysis remain available when glyph images are disabled or unavailable.

## Building from Source

Installing with Cargo requires Rust 1.88.0 or later. A release binary does not require Rust to be installed.
