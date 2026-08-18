{
  lib,
  pkg-config,
  qt6,
  rustc,
  stdenv,
}:
stdenv.mkDerivation {
  pname = "vesper-store";
  version = "0.1.0";

  src = ./.;

  nativeBuildInputs = [
    pkg-config
    qt6.wrapQtAppsHook
    rustc
  ];

  buildInputs = [
    qt6.qtbase
    qt6.qtdeclarative
    qt6.qtwayland
  ];

  buildPhase = ''
    runHook preBuild

    $CXX -std=c++20 -O2 \
      src/main.cpp \
      -o vesper-store \
      $(pkg-config --cflags --libs Qt6Core Qt6Gui Qt6Qml Qt6Quick)

    rustc --edition=2021 -C opt-level=2 \
      src/backend.rs \
      -o vesper-store-core

    runHook postBuild
  '';

  installPhase = ''
    runHook preInstall

    install -Dm755 vesper-store "$out/bin/vesper-store"
    install -Dm755 vesper-store-core "$out/libexec/vesper-store-core"
    install -Dm644 qml/Main.qml "$out/share/vesper-store/qml/Main.qml"
    install -Dm644 data/io.vesper.Store.desktop "$out/share/applications/io.vesper.Store.desktop"
    install -Dm644 data/io.vesper.Store.metainfo.xml "$out/share/metainfo/io.vesper.Store.metainfo.xml"

    runHook postInstall
  '';

  preFixup = ''
    qtWrapperArgs+=(
      --set VESPER_STORE_QML "$out/share/vesper-store/qml/Main.qml"
      --set VESPER_STORE_CORE "$out/libexec/vesper-store-core"
    )
  '';

  meta = {
    description = "Native Qt/QML application discovery surface for Vesper";
    license = lib.licenses.mit;
    mainProgram = "vesper-store";
    platforms = lib.platforms.linux;
  };
}
