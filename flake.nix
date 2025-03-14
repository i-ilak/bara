{
  inputs = {
    flake-utils.url = "https://flakehub.com/f/numtide/flake-utils/0.1.*.tar.gz";
    naersk.url = "https://flakehub.com/f/nix-community/naersk/0.1.*.tar.gz";
    nixpkgs.url = "https://flakehub.com/f/NixOS/nixpkgs/0.1.*.tar.gz";
    rust-overlay = {
      url = "https://flakehub.com/f/oxalica/rust-overlay/0.1.*.tar.gz";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    pre-commit-hooks.url = "github:cachix/git-hooks.nix";
  };

  outputs = { self, flake-utils, naersk, nixpkgs, rust-overlay, pre-commit-hooks }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        rustOverlay = final: prev: {
          rustToolchain = prev.rust-bin.stable."1.85.0".default.override {
            extensions = [ "rust-src" "rustfmt" ];
            targets = [ ];
          };
        };

        pkgs = import nixpkgs {
          inherit system;
          overlays = [
            rust-overlay.overlays.default
            (final: prev: {
              naersk = prev.callPackage naersk {
                rustc = final.rustToolchain;
                cargo = final.rustToolchain;
              };
            })
            rustOverlay
          ];
        };

        naersk' = pkgs.callPackage naersk { };
      in
      rec {
        packages = {
          bara = naersk'.buildPackage {
            src = ./.;
            preBuild = ''
              cargo clean
            '';
          };
          default = packages.bara;
        };

        apps.default = {
          type = "app";
          program = "${packages.bara}/bin/bara";
        };

        devShells = {
          default = pkgs.mkShell {
            nativeBuildInputs = with pkgs; [
              rustToolchain
              cargo-deny
              cargo-edit
              cargo-watch
              openssl
              pkg-config
              just
              nodejs_20
              pre-commit-hooks.packages.${system}.default
            ];
            RUSTUP_HOME = "/tmp/rustup";
            RUST_SRC_PATH = "${pkgs.rustToolchain}/lib/rustlib/src/rust/library";
          };
        };

        pre-commit-check = pre-commit-hooks.run {
          src = ./.;
          # If your hooks are intrusive, avoid running on each commit with a default_states like this:
          # default_stages = ["manual" "pre-push"];
          hooks = {
            elm-format.enable = true;
            clippy.enable = true;
            clippy.settings.allFeatures = true;
          };
        };
      }
    );
}
