{
  description = "a flake to build build-rs-libnix";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.11";
    fenix.url   = "github:nix-community/fenix";
    flake-utils.url = "github:numtide/flake-utils";
  };
  outputs =
  { self, nixpkgs, flake-utils, fenix } @ inputs:
    flake-utils.lib.eachDefaultSystem
      (system:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [
              fenix.overlay
            ];
          };
          build_rs_libnix = pkgs.callPackage ./default.nix {};
        in
        with pkgs;
        rec {
          # nix build (.#default, .#build_rs_libnix)
          packages = { inherit build_rs_libnix; default = build_rs_libnix; };
          devShells.default = mkShell {
            buildInputs = [
              # the toolchain used
              fenix.packages.${system}.stable.rustc
              fenix.packages.${system}.stable.cargo
              fenix.packages.${system}.stable.rust-src
              fenix.packages.${system}.stable.rustfmt
              fenix.packages.${system}.stable.clippy
            ];
          };
        }
      );
}
