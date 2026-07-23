#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# ///

"""Check that rust-toolchain.toml and Cargo.toml agree on the Rust version.

Usage: python scripts/check-rust-version.py
       chmod +x scripts/check-rust-version.py && ./scripts/check-rust-version.py
"""

import tomllib
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def toolchain_version() -> str:
    """Extract the channel from rust-toolchain.toml."""
    data = tomllib.loads((ROOT / "rust-toolchain.toml").read_text())
    return data["toolchain"]["channel"]


def cargo_msrv() -> str:
    """Extract workspace.package.rust-version from the root Cargo.toml."""
    data = tomllib.loads((ROOT / "Cargo.toml").read_text())
    return data["workspace"]["package"]["rust-version"]


def main() -> None:
    rust_toolchain_version = toolchain_version()
    cargo_toml_version = cargo_msrv()

    if rust_toolchain_version == cargo_toml_version:
        print(f"✓ rust-toolchain.toml channel = Cargo.toml rust-version = {rust_toolchain_version}")
        sys.exit(0)
    else:
        print(
            f"✗ version mismatch:\n"
            f"  rust-toolchain.toml → {rust_toolchain_version}\n"
            f"  Cargo.toml          → {cargo_toml_version}"
        )
        sys.exit(1)


if __name__ == "__main__":
    main()
