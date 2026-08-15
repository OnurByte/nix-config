{
  fetchurl,
  findutils,
  gnutar,
  gzip,
  lib,
  stdenvNoCC,
}:
stdenvNoCC.mkDerivation (finalAttrs: {
  pname = "codexbar-cli";
  version = "0.50.0";

  # Use the static musl Linux build so it does not depend on an FHS/glibc
  # loader path. Upstream publishes this artifact alongside checksums.
  src = fetchurl {
    url = "https://github.com/steipete/CodexBar/releases/download/v${finalAttrs.version}/CodexBarCLI-v${finalAttrs.version}-linux-musl-x86_64.tar.gz";
    hash = "sha256-E2hnfb7q946y2qgeZSdAT7/BkBrXZ351RcqwzYYDW4E=";
  };

  dontUnpack = true;

  nativeBuildInputs = [
    findutils
    gnutar
    gzip
  ];

  installPhase = ''
    runHook preInstall

    mkdir -p unpacked $out/bin
    tar -xzf "$src" -C unpacked

    cli="$(find unpacked \( -name CodexBarCLI -o -name codexbar \) | head -n 1)"
    test -n "$cli"
    cp -L "$cli" $out/bin/codexbar
    chmod 0755 $out/bin/codexbar

    runHook postInstall
  '';

  meta = {
    description = "Linux CLI for AI coding-provider usage and reset windows";
    homepage = "https://github.com/steipete/CodexBar";
    license = lib.licenses.mit;
    platforms = [ "x86_64-linux" ];
    mainProgram = "codexbar";
  };
})
