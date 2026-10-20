# Contribution Guide

Thank you for considering contributing. Please review the guidelines below before making a contribution. Contributions that do not follow these guidelines may not be accepted.

## Reporting Issues

Before opening an issue, check for existing issues and refer to the [FAQ](https://lusingander.github.io/sauva/faq/).

### Reporting Bugs

Please include:

- Sauva version (`sauva --version`).
- OS and terminal emulator versions, including any terminal multiplexer in use.
- Steps to reproduce, expected behavior, and actual behavior.
- Input text or code points and relevant configuration needed to reproduce the issue.

For glyph preview issues, also include the `--graphics` mode, the font family and version shown in the preview, and a screenshot if possible.

### Suggesting Features

Describe the use case and what the feature would improve. Please review the [documentation](https://lusingander.github.io/sauva/) for existing functionality first.

### Terminal Emulator Compatibility

If glyph images do not display correctly, first check whether your terminal supports the image display protocol being used. See [Compatibility](https://lusingander.github.io/sauva/getting-started/compatibility.html) for supported protocols and terminals.

## Pull Requests

Pull requests are welcome, but acceptance is not guaranteed.

### Creating Pull Requests

- Follow the relevant guidelines for [reporting issues](#reporting-issues).
- Small changes, such as documentation fixes or obvious bug fixes, can be submitted directly without an issue.
- For complex changes or fixes that alter behavior, opening an issue first is strongly recommended to discuss the approach and avoid rework.
- Keep changes focused on the pull request topic.

### Continuous Integration

[GitHub Actions](.github/workflows/build.yml) runs build, test, format, and lint checks with both stable Rust and the minimum supported Rust version defined in `Cargo.toml`.

## License

Contributions are subject to the [MIT License](LICENSE). For the bundled Unicode data, see [THIRD_PARTY_NOTICES](THIRD_PARTY_NOTICES.md) and the [Unicode License v3](UNICODE-LICENSE.txt).
