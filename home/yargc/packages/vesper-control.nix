{
  bluez,
  coreutils,
  flatpak,
  gnupatch,
  hyprland,
  inotify-tools,
  lib,
  libsecret,
  librsvg,
  libxml2,
  makeWrapper,
  networkmanager,
  qrencode,
  rustc,
  stdenv,
  systemd,
}:
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
    cp ${./vesper-icons.rs} vesper-icons.rs
    cp ${./vesper-icon-queue.rs} vesper-icon-queue.rs
    cp ${./vesper-icon-supervisor.rs} vesper-icon-supervisor.rs
    patch vesper-control.rs < ${./vesper-control-wifi-qr.patch}
    rustc --edition=2021 -C opt-level=2 vesper-control.rs -o vesper-control-core
    rustc --edition=2021 -C opt-level=2 vesper-control-router.rs -o vesper-control
    rustc --edition=2021 -C opt-level=2 vesper-icons.rs -o vesper-icon-engine-core
    rustc --edition=2021 -C opt-level=2 vesper-icon-queue.rs -o vesper-icon-queue
    rustc --edition=2021 -C opt-level=2 vesper-icon-supervisor.rs -o vesper-icon-engine
    runHook postBuild
  '';

  installPhase = ''
    runHook preInstall
    install -Dm755 vesper-control $out/bin/vesper-control
    install -Dm755 vesper-control-core $out/bin/vesper-control-core
    install -Dm755 vesper-icon-engine $out/bin/vesper-icon-engine
    install -Dm755 vesper-icon-engine-core $out/bin/vesper-icon-engine-core
    install -Dm755 vesper-icon-queue $out/bin/vesper-icon-queue

    runtimePath=${lib.makeBinPath [
      bluez
      coreutils
      flatpak
      hyprland
      inotify-tools
      libsecret
      librsvg
      libxml2
      networkmanager
      qrencode
      systemd
    ]}

    wrapProgram $out/bin/vesper-control \
      --prefix PATH : "$out/bin:$runtimePath"
    wrapProgram $out/bin/vesper-control-core \
      --prefix PATH : "$runtimePath"
    wrapProgram $out/bin/vesper-icon-engine \
      --prefix PATH : "$out/bin:$runtimePath"
    wrapProgram $out/bin/vesper-icon-engine-core \
      --prefix PATH : "$runtimePath"
    wrapProgram $out/bin/vesper-icon-queue \
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