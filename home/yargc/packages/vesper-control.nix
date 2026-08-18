{
  bluez,
  cargo,
  coreutils,
  curl,
  flatpak,
  hyprland,
  imagemagick,
  iproute2,
  jq,
  lib,
  libsecret,
  makeWrapper,
  networkmanager,
  procps,
  qrencode,
  rustc,
  snapper,
  stdenv,
  systemd,
}:
stdenv.mkDerivation {
  pname = "vesper-control";
  version = "0.8.0";

  dontUnpack = true;

  nativeBuildInputs = [ cargo makeWrapper rustc ];

  buildPhase = ''
    runHook preBuild
    cp -r ${./vesper-core} vesper-core
    chmod -R u+w vesper-core
    # Cargo modules include the canonical provider registry from the package
    # root, so materialize it before compiling the copied workspace.
    cp ${./vesper-provider-registry.rs} vesper-provider-registry.rs
    cargo build --release --locked --manifest-path vesper-core/Cargo.toml

    cp ${./vesper-control.rs} vesper-control-compat.rs
    rustc --edition=2021 -C opt-level=2 vesper-control-compat.rs -o vesper-control-compat
    runHook postBuild
  '';

  installPhase = ''
    runHook preInstall
    install -Dm755 vesper-core/target/release/vesper-control $out/bin/vesper-control
    install -Dm755 vesper-control-compat $out/bin/vesper-control-compat

    # Transitional compatibility router. First-party credential paths have
    # moved to Cargo; older commands that have not been migrated yet continue
    # to reach the frozen monolithic compatibility binary without recursion.
    cat > $out/bin/vesper-control-legacy <<EOF
#!${stdenv.shell}
set -eu
case "\''${1:-}" in
  credential|ai-status)
    exec $out/bin/vesper-control "\$@"
    ;;
  *)
    exec $out/bin/vesper-control-compat "\$@"
    ;;
esac
EOF
    chmod 0755 $out/bin/vesper-control-legacy

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
      --prefix PATH : ${lib.makeBinPath [ bluez coreutils curl flatpak hyprland imagemagick iproute2 jq libsecret networkmanager procps qrencode snapper systemd ]}

    wrapProgram $out/bin/vesper-icon-generator \
      --prefix PATH : ${lib.makeBinPath [ coreutils curl imagemagick jq ]}
    runHook postInstall
  '';

  meta = {
    description = "Native Vesper Settings control plane";
    license = lib.licenses.mit;
    mainProgram = "vesper-control";
    platforms = lib.platforms.linux;
  };
}
