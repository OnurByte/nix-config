{
  brightnessctl,
  cliphist,
  coreutils,
  hyprland,
  jq,
  lib,
  makeWrapper,
  niriScreenTime,
  power-profiles-daemon,
  rustc,
  stdenv,
  systemd,
  upower,
  wl-clipboard,
  xdg-utils,
}:
stdenv.mkDerivation {
  pname = "vesper-settings";
  version = "0.2.0";

  dontUnpack = true;

  nativeBuildInputs = [
    makeWrapper
    rustc
  ];

  buildPhase = ''
    runHook preBuild
    rustc --edition=2021 -C opt-level=2 ${./vesper-settings.rs} -o vesper-settings
    runHook postBuild
  '';

  installPhase = ''
    runHook preInstall

    install -Dm755 vesper-settings $out/bin/vesper-settings
    wrapProgram $out/bin/vesper-settings \
      --prefix PATH : ${lib.makeBinPath [
        brightnessctl
        cliphist
        coreutils
        hyprland
        jq
        niriScreenTime
        power-profiles-daemon
        systemd
        upower
        wl-clipboard
        xdg-utils
      ]}

    runHook postInstall
  '';

  meta = {
    description = "Native Rust adapter exposing desktop settings tools to Caelestia Nexus";
    license = lib.licenses.mit;
    mainProgram = "vesper-settings";
    platforms = lib.platforms.linux;
  };
}
