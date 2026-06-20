{
  rustPlatform,
  lib,
}:
let
  fs = lib.fileset;
in
rustPlatform.buildRustPackage (finalAttrs: {
  pname = "persista";
  version = "1.0.0";

  src = fs.toSource {
    root = ../.;
    fileset = fs.unions [
      ../src
      ../Cargo.lock
      ../Cargo.toml
      ../migrations
      ../.sqlx
    ];
  };

  cargoLock = {
    lockFile = ../Cargo.lock;
  };

  meta = {
    description = "API for minecraft mods saving data";
    homepage = "https://github.com/macuguita/persista";
    mainProgram = finalAttrs.pname;
    license = lib.licenses.eupl12;
    sourceProvenance = with lib.sourceTypes; [
      fromSource
    ];
  };
})