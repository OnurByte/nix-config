{
  autoPatchelfHook,
  fetchurl,
  lib,
  stdenv,
}:
stdenv.mkDerivation {
  pname = "cuprated";
  version = "0.1.0-preview";

  src = fetchurl {
    url = "https://github.com/Cuprate/cuprate/releases/download/cuprated-0.1.0-preview/cuprated-0.1.0-preview-x86_64-unknown-linux-gnu.tar.gz";
    hash = "sha256-F74UPZLEsizwIEg8itEtS2oabEqhEFoWX6iR0eEimAw=";
  };

  nativeBuildInputs = [ autoPatchelfHook ];
  buildInputs = [ stdenv.cc.cc.lib ];

  dontConfigure = true;
  dontBuild = true;

  unpackPhase = ''
    runHook preUnpack
    tar -xzf "$src"
    runHook postUnpack
  '';

  installPhase = ''
    runHook preInstall

    mkdir -p $out/bin
    candidate="$(find . -type f -name cuprated -print -quit)"
    test -n "$candidate"
    install -Dm755 "$candidate" $out/bin/cuprated

    runHook postInstall
  '';

  meta = {
    description = "Rust implementation of a Monero node";
    homepage = "https://github.com/Cuprate/cuprate";
    license = lib.licenses.agpl3Only;
    mainProgram = "cuprated";
    platforms = [ "x86_64-linux" ];
    sourceProvenance = [ lib.sourceTypes.binaryNativeCode ];
  };
}
