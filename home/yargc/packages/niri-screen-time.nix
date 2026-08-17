{
  buildGoModule,
  hyprland,
  lib,
  makeWrapper,
}:

buildGoModule {
  pname = "niri-screen-time";
  version = "0-unstable-2026-07-28";

  # Upstream ships a Nix package but is not in nixpkgs yet. Pin the exact
  # revision so the Vesper wellbeing backend stays reproducible without adding
  # another flake input just for one small CLI.
  src = builtins.fetchGit {
    url = "https://github.com/probeldev/niri-screen-time.git";
    rev = "6df60f4607a932cd9196894adf058bb1527d03ac";
  };

  # Taken from upstream package.nix for the pinned source revision.
  vendorHash = "sha256-9y1F2ZrmpiQJ9ZTq9SoRE2PxR65DDNCeBKf4M0HUQC4=";

  nativeBuildInputs = [ makeWrapper ];

  # Hyprland support shells out to `hyprctl activewindow`; make that dependency
  # explicit instead of relying on whichever PATH systemd happens to inherit.
  postInstall = ''
    wrapProgram $out/bin/niri-screen-time \
      --prefix PATH : ${lib.makeBinPath [ hyprland ]}
  '';

  meta = {
    description = "Application screen-time tracker with native Hyprland support";
    homepage = "https://github.com/probeldev/niri-screen-time";
    license = lib.licenses.mit;
    mainProgram = "niri-screen-time";
    platforms = lib.platforms.linux;
  };
}
