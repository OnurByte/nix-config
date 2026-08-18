{
  bluez,
  coreutils,
  curl,
  flatpak,
  gnupatch,
  gnutar,
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
    pname = "vesper-icons";
    version = "1.0.0";
    src = lib.cleanSource ./.;
    cargoLock.lockFile = ./Cargo.lock;
    nativeBuildInputs = [ gnupatch ];
    postPatch = ''
      patch vesper-icons.rs < ${./vesper-icons-source-guard.patch}
      patch vesper-icons.rs < ${./vesper-icons-material-axis.patch}
      patch vesper-icons.rs < ${./vesper-icons-format-support.patch}
      patch vesper-icons.rs < ${./vesper-icons-appstream-recovery.patch}
      patch vesper-icons.rs < ${./vesper-icons-grid-recipe.patch}
      patch vesper-icons.rs < ${./vesper-icons-appearance-axis.patch}
      patch vesper-icons.rs < ${./vesper-icons-state-db.patch}
      patch vesper-icons.rs < ${./vesper-icons-remote-consent.patch}
      patch vesper-icon-queue.rs < ${./vesper-icon-queue-inventory-db.patch}
      patch vesper-icon-queue.rs < ${./vesper-icon-queue-consent.patch}
      patch vesper-icon-queue.rs < ${./vesper-icon-queue-vector-semantic.patch}
      patch vesper-icon-worker.rs < ${./vesper-icon-worker-provider-defaults.patch}
      patch vesper-icon-worker.rs < ${./vesper-icon-worker-inventory-db.patch}
      patch vesper-icon-worker.rs < ${./vesper-icon-worker-validation.patch}
      patch vesper-icon-worker.rs < ${./vesper-icon-worker-consent.patch}
      patch vesper-icon-worker.rs < ${./vesper-icon-worker-vector-semantic.patch}
      patch vesper-icon-worker.rs < ${./vesper-icon-worker-export-axis.patch}
      patch vesper-icon-identity.rs < ${./vesper-icon-identity-inventory-db.patch}
    '';
    doCheck = false;
  };
in
stdenv.mkDerivation {
  pname = "vesper-control";
  version = "0.4.0";

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
    patch vesper-control.rs < ${./vesper-control-app-remove.patch}
    rustc --edition=2021 -C opt-level=2 vesper-control.rs -o vesper-control-core
    rustc --edition=2021 -C opt-level=2 vesper-control-router.rs -o vesper-control
    runHook postBuild
  '';

  installPhase = ''
    runHook preInstall
    install -Dm755 vesper-control $out/bin/vesper-control
    install -Dm755 vesper-control-core $out/bin/vesper-control-core
    install -Dm755 ${iconEngine}/bin/vesper-icon-engine $out/bin/vesper-icon-engine
    install -Dm755 ${iconEngine}/bin/vesper-icon-engine-core $out/bin/vesper-icon-engine-core
    install -Dm755 ${iconEngine}/bin/vesper-icon-queue $out/bin/vesper-icon-queue
    install -Dm755 ${iconEngine}/bin/vesper-icon-worker $out/bin/vesper-icon-worker
    install -Dm755 ${iconEngine}/bin/vesper-icon-identity $out/bin/vesper-icon-identity

    runtimePath=${lib.makeBinPath [
      bluez
      coreutils
      curl
      flatpak
      gnutar
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
      --prefix PATH : "$out/bin:$runtimePath"
    wrapProgram $out/bin/vesper-icon-engine-core \
      --prefix PATH : "$runtimePath"
    wrapProgram $out/bin/vesper-icon-queue \
      --prefix PATH : "$out/bin:$runtimePath"
    wrapProgram $out/bin/vesper-icon-worker \
      --prefix PATH : "$out/bin:$runtimePath"
    wrapProgram $out/bin/vesper-icon-identity \
      --prefix PATH : "$out/bin:$runtimePath"
    runHook postInstall
  '';

  meta = {
    description = "Native Vesper settings control plane";
    license = lib.licenses.mit;
    mainProgram = "vesper-control";
    platforms = lib.platforms.linux;
  };
}
