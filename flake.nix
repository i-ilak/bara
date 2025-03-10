{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    { self
    , flake-utils
    , naersk
    , nixpkgs
    , rust-overlay
    }:
    flake-utils.lib.eachDefaultSystem (system:
    let
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
            rust.nightly.latest.default.override {
              extensions = [ "rust-src" "rustfmt" ];
            };
      };
      pkgs = import nixpkgs {
        inherit system;
        overlays = [ rust-overlay.overlays.default rustOverlay ];
      };

      naersk' = pkgs.callPackage naersk { };

    in
    rec {
      overlays.default = rustOverlay;

      defaultPackage = naersk'.buildPackage {
        src = ./.;
      };

      devShell = pkgs.mkShell {
        nativeBuildInputs = with pkgs; [
          rustToolchain
          cargo
          openssl
          pkg-config
          cargo-deny
          cargo-edit
          cargo-watch
          rust-analyzer
          just
        ];
        env = {
          RUST_SRC_PATH = "${pkgs.rustToolchain}/lib/rustlib/src/rust/library";
        };
      };
    }
    );
}
