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
  pname = "vesper-ai-hub";
  version = "1.1.0";

  dontUnpack = true;

  nativeBuildInputs = [
    makeWrapper
    rustc
  ];

  buildPhase = ''
    runHook preBuild
    rustc --edition=2021 -C opt-level=2 ${./ai-hub.rs} -o vesper-ai-hub
    runHook postBuild
  '';

  installPhase = ''
    runHook preInstall

    install -Dm755 vesper-ai-hub $out/bin/vesper-ai-hub
    wrapProgram $out/bin/vesper-ai-hub \
      --set VESPER_AI_HUB_JQ_FILTER ${./ai-hub.jq} \
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
    description = "Native Rust data bridge for the Caelestia Vesper AI Hub";
    license = lib.licenses.mit;
    mainProgram = "vesper-ai-hub";
    platforms = lib.platforms.linux;
  };
}
