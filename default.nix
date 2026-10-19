{ pkgs, lib ? pkgs.lib }:

pkgs.rustPlatform.buildRustPackage rec {
  pname = "build-rs-libnix-cli";
  version = "0.1.12";

  # this checkout, without build outputs
  src = lib.cleanSourceWith {
    src = ./.;
    filter = path: type:
      let base = baseNameOf path;
      in !(base == "target" || base == "result" || lib.hasPrefix "result-" base || base == ".direnv");
  };

  cargoLock.lockFile = ./Cargo.lock;

  nativeBuildInputs = [];
  buildInputs = [];

  cargoBuildFlags = [ "-p cli" ];

  doCheck = true;

  meta = with lib; {
    description = "A command-line utility that extracts `--cfg` and `--check-cfg` flags from `cargo:`";
    license = with licenses; [ mit asl20 ];
    maintainers = with maintainers; [ ];
    platforms = platforms.all;
  };
}
