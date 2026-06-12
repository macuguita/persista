{
  lib,
  stdenv,
  jdk25,
  gradle_9,
  makeWrapper,
}:
let
  gradle = gradle_9;
in
stdenv.mkDerivation (finalAttrs: {
  pname = "persista";
  version = "1.0.0";

  src = ./.;

  nativeBuildInputs = [
    gradle
    jdk25
    makeWrapper
  ];

  mitmCache = gradle.fetchDeps {
    pkg = finalAttrs.finalPackage;
    data = ./deps.json;
  };

  __darwinAllowLocalNetworking = true;
  gradleFlags = [
    "-Dfile.encoding=utf-8"
    "-Dorg.gradle.java.home=${jdk25}"
  ];
  gradleBuildTask = "shadowJar";
  doCheck = true;

  installPhase = ''
    runHook preInstall
    mkdir -p $out/{bin,share/persista}
    cp build/libs/persista-${finalAttrs.version}-all.jar $out/share/persista
    makeWrapper ${lib.getExe jdk25} $out/bin/persista \
      --add-flags "-jar $out/share/persista/persista-${finalAttrs.version}-all.jar"
    runHook postInstall
  '';

  meta = {
    mainProgram = finalAttrs.pname;
    license = lib.licenses.eupl12;
    sourceProvenance = with lib.sourceTypes; [
      fromSource
      binaryBytecode
    ];
  };
})
