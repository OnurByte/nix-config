{
  appimageTools,
  fetchurl,
  lib,
  makeDesktopItem,
  symlinkJoin,
}:
let
  pname = "zcode";
  version = "3.7.5";

  src = fetchurl {
    url = "https://cdn-zcode.z.ai/zcode/electron/releases/${version}/linux-x64/ZCode-${version}-linux-x64.AppImage";
    hash = "sha256-7yV4a4x0v1I1UJc/FmKhNlnochXLh8/PxmzdWVSIuEM=";
  };

  app = appimageTools.wrapType2 {
    inherit pname version src;
  };

  desktop = makeDesktopItem {
    name = "zcode";
    desktopName = "ZCode";
    genericName = "GLM Agentic Development Environment";
    comment = "Z.AI's agentic development environment for GLM";
    exec = "zcode %U";
    icon = "applications-development";
    categories = [ "Development" ];
    startupWMClass = "ZCode";
  };
in
symlinkJoin {
  name = "${pname}-${version}";
  paths = [
    app
    desktop
  ];

  meta = {
    description = "Official ZCode agentic development environment for GLM";
    homepage = "https://zcode.z.ai/";
    license = lib.licenses.unfree;
    mainProgram = "zcode";
    platforms = [ "x86_64-linux" ];
    sourceProvenance = [ lib.sourceTypes.binaryNativeCode ];
  };
}
