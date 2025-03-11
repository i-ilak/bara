FROM nixos/nix:2.23.0

RUN echo "experimental-features = nix-command flakes" >> /etc/nix/nix.conf

RUN nix-env -iA nixpkgs.git nixpkgs.nodejs

WORKDIR /app
COPY flake.nix flake.lock ./
RUN nix build .#devShell.x86_64-linux

WORKDIR /workspace
