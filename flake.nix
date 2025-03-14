{
  inputs = {
    flake-utils.url = "https://flakehub.com/f/numtide/flake-utils/0.1.*.tar.gz";
    naersk.url = "https://flakehub.com/f/nix-community/naersk/0.1.*.tar.gz";
    nixpkgs.url = "https://flakehub.com/f/NixOS/nixpkgs/0.1.*.tar.gz";
    rust-overlay = {
      url = "https://flakehub.com/f/oxalica/rust-overlay/0.1.*.tar.gz";
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

      # Build the package
      package = naersk'.buildPackage {
        src = ./.;
        # Remove the --bin bara option since it's not a named binary target
        # Let naersk find the binary automatically
      };

    in
    rec {
      overlays.default = rustOverlay;

      # Define packages in standard flake format
      packages = {
        bara = package;
        default = package;
      };

      # Define apps with a more flexible binary detection
      apps = {
        bara = {
          type = "app";
          # Try to find the binary by checking multiple possible locations
          program =
            let
              binPath = "${package}/bin";
            in
            "${binPath}/bara";
        };
        default = apps.bara;
      };

      # Development shell remains the same
      devShell = pkgs.mkShell {
        nativeBuildInputs = with pkgs; [
          cargo
          clippy
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
