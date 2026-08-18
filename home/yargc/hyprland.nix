{ pkgs, ... }:
let
  tesseractVesper = pkgs.tesseract5.override {
    enableLanguages = [
      "eng"
      "tur"
    ];
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
  home.packages = [ vesperOcr ];

  xdg.configFile = {
    "hypr/hyprland.lua".source = ./hypr/hyprland.lua;
    "hypr/vesper/appearance.lua".source = ./hypr/appearance.lua;
    "hypr/vesper/input.lua".source = ./hypr/input.lua;
    "hypr/vesper/autostart.lua".source = ./hypr/autostart.lua;
    "hypr/vesper/binds.lua".source = ./hypr/binds.lua;
  };
}
