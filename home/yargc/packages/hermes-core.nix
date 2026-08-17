{
  callPackage,
  coreutils,
  curl,
  ghostty,
  git,
  hermesAgent,
  jq,
  lib,
  libnotify,
  makeWrapper,
  rustc,
  stdenv,
  systemd,
  writeShellApplication,
}:
let
  vesperControl = callPackage ./vesper-control.nix { };

  hermesCredentialProxy = writeShellApplication {
    name = "hermes";
    runtimeInputs = [
      coreutils
      vesperControl
    ];
    text = ''
      credential="$(${vesperControl}/bin/vesper-control consumer credential hermes)"
      if [ "$credential" = "native" ]; then
        exec ${hermesAgent}/bin/hermes "$@"
      fi
      exec ${vesperControl}/bin/vesper-control credential exec "$credential" -- \
        ${hermesAgent}/bin/hermes "$@"
    '';
  };
in
stdenv.mkDerivation {
  pname = "vesper-hermes-core";
  version = "2.1.0";

  src = ./hermes-rs;

  nativeBuildInputs = [
    makeWrapper
    rustc
  ];

  dontConfigure = true;

  buildPhase = ''
    runHook preBuild
    rustc --edition=2021 -C opt-level=2 main.rs -o vesper-hermes-core
    runHook postBuild
  '';

  installPhase = ''
    runHook preInstall

    install -Dm755 vesper-hermes-core $out/bin/vesper-hermes-core
    wrapProgram $out/bin/vesper-hermes-core \
      --set-default HERMES_RESEARCH_PROVIDER xai \
      --prefix PATH : ${lib.makeBinPath [
        coreutils
        curl
        ghostty
        git
        hermesCredentialProxy
        jq
        libnotify
        systemd
      ]}

    ln -s vesper-hermes-core $out/bin/vesper-hermes
    ln -s vesper-hermes-core $out/bin/vesper-hermes-automations
    ln -s vesper-hermes-core $out/bin/vesper-research

    runHook postInstall
  '';

  meta = {
    description = "Native Rust control plane for Vesper Hermes research and automation";
    license = lib.licenses.mit;
    mainProgram = "vesper-hermes";
    platforms = lib.platforms.linux;
  };
}
