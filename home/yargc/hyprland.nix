{ pkgs, ... }:
let
  tesseractVesper = pkgs.tesseract5.override {
    enableLanguages = [
      "eng"
      "tur"
    ];
  };

  vesperKeys = pkgs.writeShellApplication {
    name = "vesper-keys";
    runtimeInputs = [ pkgs.fzf ];
    text = ''
      cat <<'KEYS' | fzf --prompt='Vesper keys > ' --no-sort --layout=reverse --border
      Super + Return        terminal
      Super + Space         Caelestia launcher
      Super + C             control center
      Super + Shift + N     notifications
      Super + Shift + V     clipboard history
      Super + .             emoji picker
      Super + /             command palette
      Super + Shift + /     keybind cheatsheet
      Ctrl + G              Navi command -> current prompt
      Ctrl + R              Atuin fuzzy shell history
      Super + Shift + Space keyboard layout
      Alt + Tab             next window
      Alt + Shift + Tab     previous window
      Print                 screenshot
      Shift + Print         frozen region screenshot
      Super + Print         region screenshot
      Super + Ctrl + O      OCR region -> clipboard
      Super + Shift + R     region recording
      Super + Ctrl + R      region recording + audio
      Super + L             lock
      Super + Shift + Q     session menu
      Super + M             Spotify
      Super + D             Vesktop
      Super + A             ChatGPT
      Super + Shift + A     Claude Desktop
      Super + G             Grok Build
      Super + Shift + D     bb
      Super + T             T3 Code Nightly
      Super + Shift + H     Hermes Desktop
      Super + U             CodexBar
      Super + N             PychoVIM
      Super + Z             Zed
      Super + B             Zen Browser
      Super + Shift + B     Helium
      KEYS
    '';
  };

  vesperOcr = pkgs.writeShellApplication {
    name = "vesper-ocr";
    runtimeInputs = [
      pkgs.grim
      pkgs.slurp
      pkgs.wl-clipboard
      pkgs.libnotify
      tesseractVesper
    ];
    text = ''
      region="$(slurp)" || exit 0
      result="$(grim -g "$region" - | tesseract stdin stdout -l tur+eng 2>/dev/null || true)"

      if [[ -z "''${result//[[:space:]]/}" ]]; then
        notify-send "Vesper OCR" "No text found in the selected region"
        exit 0
      fi

      printf '%s' "$result" | wl-copy
      notify-send "Vesper OCR" "Recognized text copied to clipboard"
    '';
  };
in
{
  home.packages = [
    vesperKeys
    vesperOcr
  ];

  xdg.configFile = {
    "hypr/hyprland.lua".source = ./hypr/hyprland.lua;
    "hypr/vesper/appearance.lua".source = ./hypr/appearance.lua;
    "hypr/vesper/input.lua".source = ./hypr/input.lua;
    "hypr/vesper/autostart.lua".source = ./hypr/autostart.lua;
    "hypr/vesper/binds.lua".source = ./hypr/binds.lua;
  };
}
