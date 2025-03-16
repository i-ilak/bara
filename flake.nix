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

  outputs =
    { self
    , flake-utils
    , naersk
    , nixpkgs
    , rust-overlay
    , pre-commit-hooks
    }:
    flake-utils.lib.eachDefaultSystem (system:
    let
      rustOverlay = final: prev: {
        rustToolchain = prev.rust-bin.stable."1.85.0".default.override {
          extensions = [
            "rust-src"
            "rustfmt"
            "clippy"
            "rust-analyzer"
          ];
          targets = [ ];
        };
      };

      pkgs = import nixpkgs {
        inherit system;
        overlays = [
          rust-overlay.overlays.default
          rustOverlay
        ];
      };

      naerskLib = naersk.lib.${system};

      preCommitCheck = pre-commit-hooks.lib.${system}.run
        {
          src = ./.;
          hooks = {
            elm-format.enable = true;
            nixpkgs-fmt.enable = true;
            clippy = {
              enable = true;
              package = pkgs.rustToolchain;
              settings.allFeatures = true;
            };
          };
          settings = {
            rust = {
              check.cargoDeps = pkgs.rustPlatform.importCargoLock {
                lockFile = ./Cargo.lock;
              };
            };
          };
        };
    in
    rec {
      packages = {
        bara = naerskLib.buildPackage {
          pname = "bara";
          src = ./.;
          nativeBuildInputs = with pkgs; [ pkg-config ];
          buildInputs = with pkgs; [ openssl ];
          meta = with pkgs.lib; {
            description = "Your project description";
            homepage = "https://github.com/yourusername/bara";
            license = licenses.mit;
            maintainers = [ maintainers.yourusername ];
          };
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
            cargo-audit
            cargo-expand
            cargo-udeps
            cargo-nextest
            openssl
            pkg-config
            just
            nodejs_20
            pre-commit-hooks.packages.${system}.default
          ];
          RUSTUP_HOME = "/tmp/rustup";
          RUST_SRC_PATH = "${pkgs.rustToolchain}/lib/rustlib/src/rust/library";
          shellHook = ''
            ${preCommitCheck.shellHook}
          '';
        };

        release = pkgs.mkShell {
          nativeBuildInputs = devShells.default.nativeBuildInputs ++ [
            pkgs.cargo-release
          ];
          shellHook = devShells.default.shellHook;
        };
      };

      checks.pre-commit-check = preCommitCheck;
    }
    );
}
