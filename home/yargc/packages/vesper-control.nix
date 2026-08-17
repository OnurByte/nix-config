{
  bluez,
  cargo,
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
  version = "0.3.0";

  dontUnpack = true;

  nativeBuildInputs = [
    cargo
    makeWrapper
    rustc
  ];

  buildPhase = ''
    runHook preBuild

    # The Cargo control plane is the production entry point. The previous
    # single-file binary remains temporarily as a compatibility fallback while
    # domains are migrated into vesper-core modules.
    cp -r ${./vesper-core} vesper-core
    chmod -R u+w vesper-core
    cargo build --release --locked --manifest-path vesper-core/Cargo.toml

    cp ${./vesper-control.rs} vesper-control-legacy.rs
    cp ${./vesper-provider-registry.rs} vesper-provider-registry.rs
    rustc --edition=2021 -C opt-level=2 vesper-control-legacy.rs -o vesper-control-legacy

    runHook postBuild
  '';

  installPhase = ''
    runHook preInstall

    install -Dm755 vesper-core/target/release/vesper-control $out/bin/vesper-control
    install -Dm755 vesper-control-legacy $out/bin/vesper-control-legacy

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
