{ pkgs, ... }:
let
  tesseractKraken = pkgs.tesseract5.override {
    enableLanguages = [
      "eng"
      "tur"
    ];
  };

  krakenKeys = pkgs.writeShellApplication {
    name = "kraken-keys";
    runtimeInputs = [ pkgs.fzf ];
    text = ''
      cat <<'EOF' | fzf --prompt='Kraken keys > ' --no-sort --layout=reverse --border
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
      Super + Shift + D     bb
      Super + T             T3 Code
      Super + Shift + G     ZCode
      Super + Shift + H     Hermes Desktop
      Super + U             CodexBar
      Super + N             PychoVIM
      Super + Z             Zed Preview
      Super + B             Zen Browser
      Super + Shift + B     Helium
      EOF
    '';
  };

  krakenOcr = pkgs.writeShellApplication {
    name = "kraken-ocr";
    runtimeInputs = [
      pkgs.grim
      pkgs.slurp
      pkgs.wl-clipboard
      pkgs.libnotify
      tesseractKraken
    ];
    text = ''
      region="$(slurp)" || exit 0
      result="$(grim -g "$region" - | tesseract stdin stdout -l tur+eng 2>/dev/null || true)"

      if [[ -z "''${result//[[:space:]]/}" ]]; then
        notify-send "Kraken OCR" "No text found in the selected region"
        exit 0
      fi

      printf '%s' "$result" | wl-copy
      notify-send "Kraken OCR" "Recognized text copied to clipboard"
    '';
  };
in
{
  home.packages = [
    krakenKeys
    krakenOcr
  ];

  # Hyprland 0.55+ uses Lua as its primary configuration language. Keep the
  # compositor config in small files so syntax errors are isolated and CI can
  # parse the Lua independently of the Nix module.
  xdg.configFile = {
    "hypr/hyprland.lua".source = ./hypr/hyprland.lua;
    "hypr/kraken/appearance.lua".source = ./hypr/appearance.lua;
    "hypr/kraken/input.lua".source = ./hypr/input.lua;
    "hypr/kraken/autostart.lua".source = ./hypr/autostart.lua;
    "hypr/kraken/binds.lua".source = ./hypr/binds.lua;

    # hyprsunset intentionally keeps its own simple hyprlang-style config.
    "hypr/hyprsunset.conf".text = ''
      max-gamma = 100

      profile {
        time = 07:00
        identity = true
      }

      profile {
        time = 21:00
        temperature = 5000
        gamma = 0.9
      }
    '';
  };
}
