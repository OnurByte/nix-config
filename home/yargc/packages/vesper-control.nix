{
  bluez,
  cargo,
  coreutils,
  curl,
  flatpak,
  hyprland,
  imagemagick,
  jq,
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
  version = "0.7.0";

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
    install -Dm755 ${./vesper-icon-generator} $out/bin/vesper-icon-generator
    install -Dm644 ${./app-icons/manifest.txt} $out/share/vesper/app-icons/manifest.txt
    install -Dm644 ${./app-icons/zen.svg} $out/share/vesper/app-icons/zen.svg
    install -Dm644 ${./app-icons/ghostty.svg} $out/share/vesper/app-icons/ghostty.svg
    install -Dm644 ${./app-icons/thunar.svg} $out/share/vesper/app-icons/thunar.svg
    install -Dm644 ${./app-icons/vesktop.svg} $out/share/vesper/app-icons/vesktop.svg
    install -Dm644 ${./app-icons/telegram.svg} $out/share/vesper/app-icons/telegram.svg
    install -Dm644 ${./app-icons/obsidian.svg} $out/share/vesper/app-icons/obsidian.svg
    install -Dm644 ${./app-icons/session.svg} $out/share/vesper/app-icons/session.svg

    wrapProgram $out/bin/vesper-control \
      --set VESPER_CURATED_ICON_DIR $out/share/vesper/app-icons \
      --prefix PATH : ${lib.makeBinPath [
        bluez
        coreutils
        curl
        flatpak
        hyprland
        imagemagick
        jq
        libsecret
        networkmanager
        qrencode
        systemd
      ]}

    wrapProgram $out/bin/vesper-icon-generator \
      --prefix PATH : ${lib.makeBinPath [
        coreutils
        curl
        imagemagick
        jq
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
