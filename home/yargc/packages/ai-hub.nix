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
  version = "1.2.0";

  dontUnpack = true;

  nativeBuildInputs = [
    makeWrapper
    rustc
  ];

  buildPhase = ''
    runHook preBuild
    cp ${./ai-hub.rs} ai.rs
    substituteInPlace ai.rs \
      --replace-fail 'vesper-ai-hub' 'vesper-ai' \
      --replace-fail 'VESPER_AI_HUB_JQ_FILTER' 'VESPER_AI_JQ_FILTER' \
      --replace-fail 'VESPER_AI_HUB_CODEXBAR_TIMEOUT' 'VESPER_AI_CODEXBAR_TIMEOUT' \
      --replace-fail 'VESPER_AI_HUB_MAX_AGE' 'VESPER_AI_MAX_AGE' \
      --replace-fail 'AI Hub refresh lock' 'AI refresh lock'
    rustc --edition=2021 -C opt-level=2 ai.rs -o vesper-ai
    runHook postBuild
  '';

  installPhase = ''
    runHook preInstall

    install -Dm755 vesper-ai $out/bin/vesper-ai
    wrapProgram $out/bin/vesper-ai \
      --set VESPER_AI_JQ_FILTER ${./ai-hub.jq} \
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
