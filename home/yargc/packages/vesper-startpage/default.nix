{
  lib,
  makeWrapper,
  jq,
  rustc,
  sqlite,
  stdenv,
}:
let
  src = lib.fileset.toSource {
    root = ./.;
    fileset = lib.fileset.unions [
      ./server.rs
      ./web/vesper-startpage/dist
    ];
  };
in
stdenv.mkDerivation {
  pname = "vesper-startpage";
  version = "0.1.0";
  inherit src;

  nativeBuildInputs = [
    jq
    makeWrapper
    rustc
    sqlite
  ];

  dontConfigure = true;

  buildPhase = ''
    runHook preBuild
    rustc --edition=2021 -C opt-level=2 server.rs -o vesper-startpage
    runHook postBuild
  '';

  doCheck = true;
  checkPhase = ''
    runHook preCheck
    rustc --edition=2021 --test server.rs -o vesper-startpage-tests
    ./vesper-startpage-tests
    runHook postCheck
  '';

  installPhase = ''
    runHook preInstall
    install -Dm755 vesper-startpage $out/bin/vesper-startpage
    install -d $out/share/vesper-startpage
    cp -R web/vesper-startpage/dist/. $out/share/vesper-startpage/
    wrapProgram $out/bin/vesper-startpage \
      --prefix PATH : ${lib.makeBinPath [ jq sqlite ]}
    runHook postInstall
  '';

  meta = {
    description = "Local Vesper startpage for Zen, Helium, Hermes and Tor";
    license = lib.licenses.mit;
    mainProgram = "vesper-startpage";
    platforms = lib.platforms.linux;
  };
}
