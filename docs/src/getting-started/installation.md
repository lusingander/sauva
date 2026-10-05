# Installation

## Cargo

Install from [crates.io](https://crates.io/crates/sauva):

```sh
cargo install --locked sauva
```

See [Requirements](./requirements.md) for the minimum Rust version.

## Homebrew (macOS)

```sh
brew install lusingander/tap/sauva
```

The formula is available in [lusingander/homebrew-tap](https://github.com/lusingander/homebrew-tap/blob/master/sauva.rb).

## Release Binaries

Download an archive for your operating system and architecture from the [releases page](https://github.com/lusingander/sauva/releases).

| Operating system | Architecture | Target |
| --- | --- | --- |
| macOS | Apple Silicon | `aarch64-apple-darwin` |
| macOS | Intel | `x86_64-apple-darwin` |
| Linux | ARM64 | `aarch64-unknown-linux-gnu` or `aarch64-unknown-linux-musl` |
| Linux | x86-64 | `x86_64-unknown-linux-gnu` or `x86_64-unknown-linux-musl` |

Extract the archive and put the `sauva` executable in a directory on your `PATH`.

## Verify the Installation

```sh
sauva --version
```

Continue with [Basic Usage](./basic-usage.md).
