{
  lib,
  stdenvNoCC,
  makeWrapper,
  gcc,
  bash,
  coreutils,
  jq,
  openssl,
  lsof,
  procps,
  gnugrep,
  python3,
  gtk4,
  gtk4-layer-shell,
  libadwaita,
  src,
  codexbar,
}:
let
  python = python3.withPackages (ps: [ ps.pygobject3 ]);
  runtimePath = lib.makeBinPath [
    bash
    coreutils
    jq
    openssl
    lsof
    procps
    gnugrep
  ];
  typelibPath = lib.makeSearchPath "lib/girepository-1.0" [
    gtk4
    gtk4-layer-shell
    libadwaita
  ];
in
stdenvNoCC.mkDerivation {
  pname = "codexbar-wayland-ui";
  version = "unstable";
  inherit src;

  nativeBuildInputs = [
    gcc
    makeWrapper
  ];

  buildPhase = ''
    runHook preBuild
    gcc -shared -fPIC -O2 cert_redirect.c -o cert_redirect.so -ldl
    runHook postBuild
  '';

  installPhase = ''
    runHook preInstall

    app=$out/share/codexbar-wayland-ui
    mkdir -p "$app" "$out/bin" "$out/share/codexbar-waybar/icons"

    install -m 0755 codexbar.sh "$app/codexbar.sh"
    install -m 0755 codexbar-popup.py "$app/codexbar-popup.py"
    install -m 0755 cert_redirect.so "$app/cert_redirect.so"
    install -m 0644 assets/providers/ProviderIcon-*.svg "$out/share/codexbar-waybar/icons/"
    if [ -f assets/providers/NOTICE ]; then
      install -m 0644 assets/providers/NOTICE "$out/share/codexbar-waybar/icons/NOTICE"
    fi

    makeWrapper ${bash}/bin/bash "$out/bin/codexbar-status" \
      --add-flags "$app/codexbar.sh" \
      --set CODEXBAR_BIN ${lib.getExe codexbar} \
      --prefix PATH : ${runtimePath}

    makeWrapper ${python}/bin/python3 "$out/bin/codexbar-popup" \
      --add-flags "$app/codexbar-popup.py" \
      --set CODEXBAR_BIN ${lib.getExe codexbar} \
      --set CODEXBAR_LAYER_SHELL_LIB ${gtk4-layer-shell}/lib/libgtk4-layer-shell.so \
      --prefix GI_TYPELIB_PATH : ${typelibPath} \
      --prefix PATH : ${runtimePath}

    runHook postInstall
  '';

  meta = {
    description = "Wayland GTK4 usage popover for the CodexBar Linux CLI";
    homepage = "https://github.com/Marouan-chak/codexbar-waybar";
    license = lib.licenses.mit;
    platforms = lib.platforms.linux;
    mainProgram = "codexbar-popup";
  };
}
