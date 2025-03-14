{
  inputs = {
  flake-utils.url = "github:numtide/flake-utils";
  naersk.url = "github:nix-community/naersk";
  nixpkgs.url = "github:NixOS/nixpkgs";
};

  outputs =
    { self
    , flake-utils
    , naersk
    , nixpkgs
    }:
    flake-utils.lib.eachDefaultSystem (system:
    let
      pkgs = import nixpkgs {
        inherit system;
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
      };
    }
    );
}
