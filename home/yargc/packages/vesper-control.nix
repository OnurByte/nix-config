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
    cp ${./vesper-provider-registry.rs} vesper-provider-registry.rs
    patch vesper-control.rs < ${./vesper-control-provider-registry.patch}
    patch vesper-control.rs < ${./vesper-control-proxy-hardening.patch}
    patch vesper-control.rs < ${./vesper-control-wifi-qr.patch}
    patch vesper-control.rs < ${./vesper-control-wellbeing.patch}
    patch vesper-control.rs < ${./vesper-control-app-permissions.patch}
    patch vesper-control.rs < ${./vesper-control-credential-aliases.patch}
    patch vesper-control.rs < ${./vesper-control-exec-separator.patch}
    patch vesper-control.rs < ${./vesper-control-wellbeing-toggle.patch}
    rustc --edition=2021 -C opt-level=2 vesper-control.rs -o vesper-control
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
    description = "Native Vesper Settings control plane";
    license = lib.licenses.mit;
    mainProgram = "vesper-control";
    platforms = lib.platforms.linux;
  };
}
