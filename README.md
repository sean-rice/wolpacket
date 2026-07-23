# wolpacket

Wake-On-LAN command and TUI: wake devices on your LAN.

## Usage

```sh
# Interactive TUI
wolpacket

# Headless: send a magic packet directly
wolpacket --device sonos-move2
```

## Development

Requires [Nix](https://nixos.org/) with flakes enabled.

```sh
direnv allow          # enter the dev shell
cargo build           # debug build
cargo test            # run tests
nix build             # optimized release build
nix run               # run the release binary
```

### QA checks

```sh
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo audit
cargo deny check
```
