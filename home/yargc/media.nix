{
  inputs,
  pkgs,
  ...
}:
let
  spicePkgs = inputs.spicetify-nix.legacyPackages.${pkgs.stdenv.system};
in
{
  programs = {
    spicetify = {
      enable = true;

      # Keep Spotify familiar but remove the noise that makes the stock client
      # annoying to use. The module supplies Spotify itself; do not add pkgs.spotify.
      enabledExtensions = with spicePkgs.extensions; [
        adblockify
        hidePodcasts
        shuffle
      ];

      theme = spicePkgs.themes.catppuccin;
      colorScheme = "mocha";
    };

    # Local audio/video player. MPRIS makes active MPV sessions visible to
    # Caelestia and hardware media controls instead of creating another player UI.
    mpv = {
      enable = true;
      scripts = [ pkgs.mpvScripts.mpris ];
      config = {
        vo = "gpu-next";
        hwdec = "auto-safe";
        ao = "pipewire";
        save-position-on-quit = true;
        keep-open = "yes";
      };
    };
  };
}
