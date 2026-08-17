{
  agentCockpit,
  codexbar,
  coreutils,
  jq,
  lib,
  makeWrapper,
  privacyHud,
  rustc,
  stdenv,
}:
stdenv.mkDerivation {
  pname = "vesper-ai";
  version = "1.3.0";

  dontUnpack = true;

  nativeBuildInputs = [
    makeWrapper
    rustc
  ];

  buildPhase = ''
    runHook preBuild
    cp ${./ai.rs} ai.rs
    rustc --edition=2021 -C opt-level=2 ai.rs -o vesper-ai
    runHook postBuild
  '';

  installPhase = ''
    runHook preInstall

    install -Dm755 vesper-ai $out/bin/vesper-ai
    wrapProgram $out/bin/vesper-ai \
      --set VESPER_AI_JQ_FILTER ${./ai.jq} \
      --prefix PATH : ${lib.makeBinPath [
        coreutils
        jq
        codexbar
        agentCockpit
        privacyHud
      ]}

    runHook postInstall
  '';

  meta = {
    description = "Native Rust data bridge for Vesper AI";
    license = lib.licenses.mit;
    mainProgram = "vesper-ai";
    platforms = lib.platforms.linux;
  };
}
