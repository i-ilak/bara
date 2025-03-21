{ pkgs
, ...
}:
{
  projectRootFile = "flake.nix";
  programs = {
    just.enable = true;
    nixpkgs-fmt.enable = true;
    rustfmt.enable = true;
    toml-sort.enable = true;
    mdformat.enable = true;
  };
  settings.global.excludes = [
    ".envrc"
    ".github/workflows/*.yml"
  ];
}
