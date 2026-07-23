{
  description = "Rust development environment for `wolpacket`";

  inputs = {
    # nixpkgs.url = "https://flakehub.com/f/NixOS/nixpkgs/0.1.*.tar.gz";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, rust-overlay }:
    let
      supportedSystems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];

      # Read metadata from Cargo.toml so the flake stays in sync automatically.
      workspaceToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
      binToml = builtins.fromTOML (builtins.readFile ./wolpacket/Cargo.toml);
      pname = binToml.package.name;
      version = workspaceToml.workspace.package.version;

      rustOverlay = final: prev: {
        rustToolchain =
          let
            rust = prev.rust-bin;
          in
          if builtins.pathExists ./rust-toolchain.toml then
            rust.fromRustupToolchainFile ./rust-toolchain.toml
          else if builtins.pathExists ./rust-toolchain then
            rust.fromRustupToolchainFile ./rust-toolchain
          else
            rust.stable.latest.default.override {
              extensions = [ "rust-src" "rustfmt" ];
            };
      };

      forEachSupportedSystem = f: nixpkgs.lib.genAttrs supportedSystems (system: f {
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default rustOverlay ];
        };
      });
    in
    {
      overlays.default = rustOverlay;

      packages = forEachSupportedSystem ({ pkgs }: let
        rustPlatform = pkgs.makeRustPlatform {
          cargo = pkgs.rustToolchain;
          rustc = pkgs.rustToolchain;
        };
      in {
        default = rustPlatform.buildRustPackage {
          inherit pname version;
          src = ./. ;
          cargoLock.lockFile = ./Cargo.lock;

          nativeBuildInputs = [];
          buildInputs = [];

          # Optimize for binary size
          CARGO_PROFILE_RELEASE_OPT_LEVEL = "s";
          CARGO_PROFILE_RELEASE_LTO = "true";
          CARGO_PROFILE_RELEASE_CODEGEN_UNITS = "1";

          doCheck = true;
        };
      });

      devShells = forEachSupportedSystem ({ pkgs }: {
        default = pkgs.mkShell {
          packages = with pkgs; [
            rustToolchain
            cargo-audit
            cargo-deny
            cargo-edit
            cargo-expand
            rust-analyzer
            bacon

            # Binary size analysis
            cargo-bloat
            cargo-binutils

            # If the dependencies need system libs, you usually need pkg-config + the lib
            #pkg-config
            #openssl
            #libiconv

            zsh
          ];

          env = {
            RUST_BACKTRACE = "full";
            # Required by rust-analyzer
            RUST_SRC_PATH = "${pkgs.rustToolchain}/lib/rustlib/src/rust/library";
          };
        };
      });
    };
}
