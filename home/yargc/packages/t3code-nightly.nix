{
  appimageTools,
  fetchurl,
  gh,
  git,
  lib,
  makeDesktopItem,
  makeWrapper,
}:
let
  pname = "t3code-nightly";
  version = "0.0.34-nightly.20260816.1105";

  src = fetchurl {
    url = "https://github.com/pingdotgg/t3code/releases/download/v${version}/T3-Code-${version}-x86_64.AppImage";
    hash = "sha256-KhUDLKg4wbX9FCjnoPkxk3wVfKlygXpvgj8cmujQXfU=";
  };

  desktopItem = makeDesktopItem {
    name = "t3code";
    desktopName = "T3 Code Nightly";
    comment = "Nightly desktop control surface for local coding agents";
    exec = "${pname} %U";
    icon = "applications-development";
    terminal = false;
    startupWMClass = "t3code";
    categories = [ "Development" ];
    mimeTypes = [ "x-scheme-handler/t3code" ];
  };
in
appimageTools.wrapType2 {
  inherit pname version src;

  nativeBuildInputs = [ makeWrapper ];

  extraInstallCommands = ''
    install -Dm444 \
      ${desktopItem}/share/applications/t3code.desktop \
      $out/share/applications/t3code.desktop

    # Coding-agent CLIs are already provided by the Home Manager profile.
    # Preserve the user's PATH so T3 discovers Codex, Claude Code and OpenCode
    # without coupling this standalone nightly package to unfree agent derivations.
    wrapProgram $out/bin/${pname} \
      --prefix PATH : ${lib.makeBinPath [
        gh
        git
      ]}
  '';

  meta = {
    description = "Nightly desktop control surface for local coding agents";
    homepage = "https://github.com/pingdotgg/t3code";
    license = lib.licenses.mit;
    mainProgram = pname;
    platforms = [ "x86_64-linux" ];
    sourceProvenance = [ lib.sourceTypes.binaryNativeCode ];
  };
}
