{
  rustPlatform,
  lib,
}:
rustPlatform.buildRustPackage (finalAttrs: {
  pname = "persista";
  version = "1.0.0";
  src = ../.;

  cargoLock = {
    lockFile = ../Cargo.lock;
  };

  meta = {
    mainProgram = finalAttrs.pname;
    license = lib.licenses.eupl12;
    sourceProvenance = with lib.sourceTypes; [
      fromSource
      binaryBytecode
    ];
  };
})
