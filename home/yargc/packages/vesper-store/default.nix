{
  lib,
  pkg-config,
  qt6,
  rustc,
  sqlite,
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
    sqlite
  ];

  buildInputs = [
    qt6.qtbase
    qt6.qtdeclarative
    qt6.qtwayland
  ];

  buildPhase = ''
    runHook preBuild

    $CXX -std=c++20 -O2 -fPIC \
      src/main.cpp \
      -o vesper-store \
      $(pkg-config --cflags --libs Qt6Core Qt6Gui Qt6Qml Qt6Quick)

    rustc --edition=2021 -C opt-level=2 \
      src/backend.rs \
      -o vesper-store-core

    runHook postBuild
  '';

  doCheck = true;
  checkPhase = ''
    runHook preCheck

    ./vesper-store-core sources \
      | grep -F '"flathub":{"enabled":false' >/dev/null
    env -u VESPER_STORE_CATALOG ./vesper-store-core catalog-status \
      | grep -F '"available":false' >/dev/null

    fixture="$TMPDIR/catalog.sqlite"
    sqlite3 "$fixture" < data/catalog-schema.sql
    VESPER_STORE_CATALOG="$fixture" ./vesper-store-core catalog-status \
      | grep -F '"available":true' >/dev/null

    runHook postCheck
  '';

  installPhase = ''
    runHook preInstall

    install -Dm755 vesper-store "$out/bin/vesper-store"
    install -Dm755 vesper-store-core "$out/libexec/vesper-store-core"
    install -Dm644 qml/Main.qml "$out/share/vesper-store/qml/Main.qml"
    install -Dm644 data/catalog-schema.sql "$out/share/vesper-store/catalog-schema.sql"
    install -Dm644 data/io.vesper.Store.desktop "$out/share/applications/io.vesper.Store.desktop"
    install -Dm644 data/io.vesper.Store.metainfo.xml "$out/share/metainfo/io.vesper.Store.metainfo.xml"

    runHook postInstall
  '';

  preFixup = ''
    qtWrapperArgs+=(
      --set VESPER_STORE_QML "$out/share/vesper-store/qml/Main.qml"
      --set VESPER_STORE_CORE "$out/libexec/vesper-store-core"
      --prefix PATH : "${lib.makeBinPath [ sqlite ]}"
    )
  '';

  meta = {
    description = "Native Qt/QML application discovery surface for Vesper";
    license = lib.licenses.mit;
    mainProgram = "vesper-store";
    platforms = lib.platforms.linux;
  };
}
