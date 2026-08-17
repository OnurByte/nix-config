{
  bluez,
  lib,
  makeWrapper,
  networkmanager,
  qrencode,
  rustc,
  stdenv,
  systemd,
}:
stdenv.mkDerivation {
  pname = "vesper-network";
  version = "0.1.0";

  dontUnpack = true;

  nativeBuildInputs = [
    makeWrapper
    rustc
  ];

  buildPhase = ''
    runHook preBuild
    rustc --edition=2021 -C opt-level=2 ${./vesper-network.rs} -o vesper-network
    runHook postBuild
  '';

  installPhase = ''
    runHook preInstall
    install -Dm755 vesper-network $out/bin/vesper-network
    wrapProgram $out/bin/vesper-network \
      --prefix PATH : ${lib.makeBinPath [
        bluez
        networkmanager
        qrencode
        systemd
      ]}
    runHook postInstall
  '';

  meta = {
    description = "Native Vesper network settings controller";
    license = lib.licenses.mit;
    mainProgram = "vesper-network";
    platforms = lib.platforms.linux;
  };
}
