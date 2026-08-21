{
  coreutils,
  curl,
  ffmpeg_8,
  file,
  jq,
  lib,
  makeWrapper,
  rustc,
  sqlite,
  stdenv,
}:
stdenv.mkDerivation {
  pname = "vesper-xpatla";
  version = "0.1.0";

  dontUnpack = true;

  nativeBuildInputs = [
    makeWrapper
    rustc
  ];

  buildPhase = ''
    runHook preBuild
    rustc --edition=2021 -C opt-level=2 ${./vesper-xpatla.rs} -o vesper-xpatla
    runHook postBuild
  '';

  installPhase = ''
    runHook preInstall
    install -Dm755 vesper-xpatla $out/bin/vesper-xpatla
    wrapProgram $out/bin/vesper-xpatla \
      --prefix PATH : ${lib.makeBinPath [
        coreutils
        curl
        ffmpeg_8
        file
        jq
        sqlite
      ]}
    runHook postInstall
  '';

  meta = {
    description = "Dynamic FxTwitter news ingestion and media provenance bridge for Vesper";
    license = lib.licenses.mit;
    mainProgram = "vesper-xpatla";
    platforms = lib.platforms.linux;
  };
}
