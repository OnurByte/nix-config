{
  buildNpmPackage,
  fetchFromGitHub,
  lib,
  nodejs_24,
}:
buildNpmPackage rec {
  pname = "turnlens";
  version = "0.4.0";

  src = fetchFromGitHub {
    owner = "kelesmert";
    repo = "turnlens";
    tag = "v${version}";
    hash = "sha256-fv475/OTTr9WxHi72t/CSyvnjwLLvk8UWeRgkQUT1lw=";
  };

  npmDepsHash = "sha256-UFF32LJNWnAwpmLspRYJODRxkEG9hUpec4A9Af9jNjU=";
  nodejs = nodejs_24;

  meta = {
    description = "Per-turn token and API-equivalent cost monitoring for Codex and Claude Code";
    homepage = "https://github.com/kelesmert/turnlens";
    license = lib.licenses.mit;
    mainProgram = "turnlens";
    platforms = lib.platforms.linux;
  };
}
