{
  inputs = {
    flake-utils.url = "https://flakehub.com/f/numtide/flake-utils/0.1.102.tar.gz";
    naersk.url = "https://flakehub.com/f/nix-community/naersk/0.1.353.tar.gz";
    nixpkgs.url = "https://flakehub.com/f/NixOS/nixpkgs/0.1.764837.tar.gz";
    rust-overlay = {
      url = "https://flakehub.com/f/oxalica/rust-overlay/0.1.1715.tar.gz";
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
          nodejs_20
        ];
        env = {
          RUST_SRC_PATH = "${pkgs.rustToolchain}/lib/rustlib/src/rust/library";
        };
      };
    }
    );
}
