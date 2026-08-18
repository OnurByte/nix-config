{
  bluez,
  coreutils,
  curl,
  flatpak,
  gnupatch,
  gnutar,
  gzip,
  gtk3,
  hyprland,
  imagemagick,
  inotify-tools,
  jq,
  lib,
  libsecret,
  librsvg,
  libxml2,
  makeWrapper,
  networkmanager,
  qrencode,
  rustc,
  rustPlatform,
  sqlite,
  stdenv,
  systemd,
}:
let
  iconEngine = rustPlatform.buildRustPackage {
    pname = "vesper-icon-engine";
    version = "0.3.0";
    src = ./vesper-icons;

    cargoLock.lockFile = ./vesper-icons/Cargo.lock;
    buildInputs = [ curl ];

    meta = {
      description = "Vesper adaptive application icon engine";
      license = lib.licenses.mit;
      mainProgram = "vesper-icon-engine";
      platforms = lib.platforms.linux;
    };
  };
in
stdenv.mkDerivation {
  pname = "vesper-control";
  version = "0.3.0";

  dontUnpack = true;

  nativeBuildInputs = [
    gnupatch
    makeWrapper
    rustc
  ];

  buildPhase = ''
    runHook preBuild
    cp ${./vesper-control.rs} vesper-control.rs
    cp ${./vesper-control-router.rs} vesper-control-router.rs
    patch vesper-control.rs < ${./vesper-control-wifi-qr.patch}
    rustc --edition=2021 -C opt-level=2 vesper-control.rs -o vesper-control-core
    rustc --edition=2021 -C opt-level=2 vesper-control-router.rs -o vesper-control
    runHook postBuild
  '';

  installPhase = ''
    runHook preInstall
    install -Dm755 vesper-control $out/bin/vesper-control
    install -Dm755 vesper-control-core $out/bin/vesper-control-core
    install -Dm755 ${iconEngine}/bin/vesper-icon-engine $out/bin/vesper-icon-engine

    runtimePath=${lib.makeBinPath [
      bluez
      coreutils
      flatpak
      gnutar
      gzip
      gtk3
      hyprland
      imagemagick
      inotify-tools
      jq
      libsecret
      librsvg
      libxml2
      networkmanager
      qrencode
      sqlite
      systemd
    ]}

    wrapProgram $out/bin/vesper-control \
      --prefix PATH : "$out/bin:$runtimePath"
    wrapProgram $out/bin/vesper-control-core \
      --prefix PATH : "$runtimePath"
    wrapProgram $out/bin/vesper-icon-engine \
      --prefix PATH : "$runtimePath"
    runHook postInstall
  '';

  meta = {
    description = "Native Vesper settings control plane";
    license = lib.licenses.mit;
    mainProgram = "vesper-control";
    platforms = lib.platforms.linux;
  };
}
