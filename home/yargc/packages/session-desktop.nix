{
  appimageTools,
  fetchurl,
  lib,
  makeDesktopItem,
}:
let
  pname = "session-desktop";
  version = "1.18.0";

  # nixpkgs' source build for 1.18.0 currently fails because the upstream
  # pnpm lockfile omits integrity for the session-emoji-mart tarball. Use the
  # official upstream Linux AppImage for the same release until that packaging
  # path is fixed.
  src = fetchurl {
    url = "https://github.com/session-foundation/session-desktop/releases/download/v${version}/session-desktop-linux-x86_64-${version}.AppImage";
    hash = "sha256-9YMxisgjTVby3v2bxE1XuC0g/haAUt7DhSgEi2lPxHY=";
  };

  desktopItem = makeDesktopItem {
    name = pname;
    desktopName = "Session";
    comment = "Onion routing based messenger";
    exec = "${pname} %U";
    icon = "internet-chat";
    terminal = false;
    startupWMClass = "Session";
    categories = [
      "Network"
      "InstantMessaging"
      "Chat"
    ];
  };
in
appimageTools.wrapType2 {
  inherit pname version src;

  extraInstallCommands = ''
    install -Dm444 \
      ${desktopItem}/share/applications/${pname}.desktop \
      $out/share/applications/${pname}.desktop
  '';

  meta = {
    description = "Onion routing based messenger";
    homepage = "https://getsession.org/";
    changelog = "https://github.com/session-foundation/session-desktop/releases/tag/v${version}";
    license = lib.licenses.gpl3Only;
    mainProgram = pname;
    platforms = [ "x86_64-linux" ];
    sourceProvenance = [ lib.sourceTypes.binaryNativeCode ];
  };
}
