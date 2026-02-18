# Cosmic Calculator

A calculator application for the [COSMIC][cosmic] desktop environment, built with [libcosmic][libcosmic] and Rust.

## Features

- **Basic mode** — standard arithmetic with a button-grid UI: addition, subtraction, multiplication, division, parentheses, sign toggle, and percentage
- **Factorial** — integer and floating-point factorial via the `!` operator, with gamma function support for non-integers
- **Expression history** — scrollable list of past calculations with copy-to-input support
- **Keyboard input** — type expressions directly or use the on-screen buttons
- **Advanced and Developer modes** — placeholder pages for future functionality

### Architecture

The project is split into two crates:

| Crate | Purpose |
|---|---|
| `cosmic-calculator` (root) | COSMIC/Iced GUI application |
| `calclib` | Standalone expression engine — lexer, parser, AST, and evaluator with no GUI dependencies |

## Building

Requires a Rust toolchain (install via [rustup][rustup]). A [justfile](./justfile) is provided for the [just][just] command runner:

| Command | Description |
|---|---|
| `just` | Build the release binary (default recipe) |
| `just run` | Build and run the application |
| `just build-debug` | Build with the debug profile |
| `just test` | Run all tests across the workspace |
| `just test-app` | Run only the app crate tests |
| `just test-lib` | Run only the `calclib` tests |
| `just test-watch` | Continuously run `calclib` unit tests with [bacon][bacon] |
| `just check` | Run clippy with pedantic warnings |
| `just check-json` | Clippy with JSON output for IDE/LSP integration |
| `just clean` | Run `cargo clean` |

## Installation

```sh
just build-release
just install
```

Override paths with `rootdir` and `prefix`:

```sh
just rootdir=/tmp/staging prefix=/usr install
```

To uninstall:

```sh
just uninstall
```

## Packaging

For distribution packaging, vendor dependencies and build offline:

```sh
just vendor
just build-vendored
just rootdir=debian/cosmic-calculator prefix=/usr install
```

It is recommended to create the vendored tarball on the host system before entering a sandboxed build environment.

## Translators

[Fluent][fluent] is used for localization. Translation files live in the [i18n directory](./i18n). To add a new language:

1. Copy the [English (en) localization](./i18n/en) directory
2. Rename it to the target [ISO 639-1 language code][iso-codes]
3. Translate each [message identifier][fluent-guide] (messages without a translation can be omitted)

## Development

Install [rustup][rustup] and configure your editor with [rust-analyzer][rust-analyzer]. Optional speed-ups:

- Disable LTO in the release profile
- Install the [mold][mold] linker (only benefits builds with LTO disabled)
- Use [sccache][sccache] for compilation caching

Run the `calclib` tests:

```sh
cargo test -p calclib
```

## License

[MIT](./LICENSE)

[bacon]: https://github.com/Canop/bacon
[cosmic]: https://system76.com/cosmic
[fluent]: https://projectfluent.org/
[fluent-guide]: https://projectfluent.org/fluent/guide/hello.html
[iso-codes]: https://en.wikipedia.org/wiki/List_of_ISO_639-1_codes
[just]: https://github.com/casey/just
[libcosmic]: https://github.com/pop-os/libcosmic
[mold]: https://github.com/rui314/mold
[rust-analyzer]: https://rust-analyzer.github.io/
[rustup]: https://rustup.rs/
[sccache]: https://github.com/mozilla/sccache
