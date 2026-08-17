{
  bluez,
  coreutils,
  flatpak,
  gnupatch,
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
    gnupatch
    makeWrapper
    rustc
  ];

  buildPhase = ''
    runHook preBuild
    cp ${./vesper-control.rs} vesper-control.rs
    patch vesper-control.rs < ${./vesper-control-wifi-qr.patch}
    patch vesper-control.rs < ${./vesper-control-wellbeing.patch}
    rustc --edition=2021 -C opt-level=2 vesper-control.rs -o vesper-control
    runHook postBuild
  '';

  installPhase = ''
    runHook preBuild
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
    runHook postBuild
  '';

  meta = {
    description = "Native Vesper settings control plane";
    license = lib.licenses.mit;
    mainProgram = "vesper-control";
    platforms = lib.platforms.linux;
  };
}
