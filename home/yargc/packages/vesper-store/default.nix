{
  lib,
  pkg-config,
  qt6,
  jq,
  rustc,
  sqlite,
  stdenv,
}:
stdenv.mkDerivation {
  pname = "vesper-store";
  version = "0.1.0";

  src = ./.;

  nativeBuildInputs = [
    jq
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
    printf '%s\n' '{"schemaVersion":1,"system":"${stdenv.hostPlatform.system}","nixpkgsRevision":"0000000000000000000000000000000000000000","generatedAt":"2026-01-01T00:00:00Z"}' \
      > "$TMPDIR/catalog-meta.json"
    VESPER_STORE_EXPECTED_SYSTEM="${stdenv.hostPlatform.system}" \
      VESPER_STORE_CATALOG="$fixture" ./vesper-store-core catalog-status \
      | grep -F '"available":true' >/dev/null

    incomplete="$TMPDIR/catalog-incomplete.sqlite"
    sqlite3 "$incomplete" < data/catalog-schema.sql
    sqlite3 "$incomplete" 'DROP TABLE aliases;'
    VESPER_STORE_EXPECTED_SYSTEM="${stdenv.hostPlatform.system}" \
      VESPER_STORE_CATALOG="$incomplete" VESPER_STORE_CATALOG_META="$TMPDIR/catalog-meta.json" \
      ./vesper-store-core catalog-status | grep -F '"available":false' >/dev/null

    invalid_meta="$TMPDIR/catalog-meta-invalid.json"
    printf '%s\n' '{"schemaVersion":1,"system":"x86_64-linux","nixpkgsRevision":"not-a-revision","generatedAt":"2026-01-01T00:00:00Z"}' \
      > "$invalid_meta"
    VESPER_STORE_EXPECTED_SYSTEM="${stdenv.hostPlatform.system}" \
      VESPER_STORE_CATALOG="$fixture" VESPER_STORE_CATALOG_META="$invalid_meta" \
      ./vesper-store-core catalog-status | grep -F '"available":false' >/dev/null

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
      --set VESPER_STORE_EXPECTED_SYSTEM "${stdenv.hostPlatform.system}"
      --prefix PATH : "${lib.makeBinPath [ jq sqlite ]}"
    )
  '';

  meta = {
    description = "Native Qt/QML application discovery surface for Vesper";
    license = lib.licenses.mit;
    mainProgram = "vesper-store";
    platforms = lib.platforms.linux;
  };
}
