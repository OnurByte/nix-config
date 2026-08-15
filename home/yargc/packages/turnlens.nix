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
    hash = lib.fakeHash;
  };

  npmDepsHash = lib.fakeHash;
  nodejs = nodejs_24;

  meta = {
    description = "Per-turn token and API-equivalent cost monitoring for Codex and Claude Code";
    homepage = "https://github.com/kelesmert/turnlens";
    license = lib.licenses.mit;
    mainProgram = "turnlens";
    platforms = lib.platforms.linux;
  };
}
