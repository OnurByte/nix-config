{
  bluez,
  coreutils,
  flatpak,
  hyprland,
  lib,
  libsecret,
  makeWrapper,
  networkmanager,
  qrencode,
  rustc,
  stdenv,
  systemd,
}:
stdenv.mkDerivation {
  pname = "vesper-control";
  version = "0.1.0";

  dontUnpack = true;

  nativeBuildInputs = [
    makeWrapper
    rustc
  ];

  buildPhase = ''
    runHook preBuild
    rustc --edition=2021 -C opt-level=2 ${./vesper-control.rs} -o vesper-control
    runHook postBuild
  '';

  installPhase = ''
    runHook preInstall
    install -Dm755 vesper-control $out/bin/vesper-control
    wrapProgram $out/bin/vesper-control \
      --prefix PATH : ${lib.makeBinPath [
        bluez
        coreutils
        flatpak
        hyprland
        libsecret
        networkmanager
        qrencode
        systemd
      ]}
    runHook postInstall
  '';

  meta = {
    description = "Native Vesper settings control plane";
    license = lib.licenses.mit;
    mainProgram = "vesper-control";
    platforms = lib.platforms.linux;
  };
}
