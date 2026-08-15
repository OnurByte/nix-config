{
  inputs,
  pkgs,
  ...
}:
let
  zedPreview = pkgs.writeShellApplication {
    name = "zed-preview";
    runtimeInputs = with pkgs; [
      curl
      coreutils
      gnutar
      gzip
      xz
      gnused
    ];
    text = ''
      set -euo pipefail

      zed="$HOME/.local/zed-preview.app/bin/zed"
      if [ ! -x "$zed" ]; then
        echo "Installing Zed Preview from the official Zed installer..." >&2
        curl -fsSL https://zed.dev/install.sh | ZED_CHANNEL=preview sh
      fi

      exec "$zed" "$@"
    '';
  };

  hermes = pkgs.writeShellApplication {
    name = "hermes-bootstrap";
    runtimeInputs = with pkgs; [
      bash
      curl
      coreutils
      git
      gnugrep
      gnused
      gawk
      findutils
      gnutar
      gzip
      unzip
      nodejs_24
      python3
      uv
      ripgrep
      ffmpeg
    ];
    text = ''
      set -euo pipefail

      hermes_bin="$HOME/.local/bin/hermes"
      if [ ! -x "$hermes_bin" ]; then
        echo "Installing Hermes Agent from Nous Research..." >&2
        curl -fsSL https://hermes-agent.nousresearch.com/install.sh | bash -s -- --skip-browser
      fi

      exec "$hermes_bin" "$@"
    '';
  };
in
{
  home.packages = [
    inputs.codexbar.packages.${pkgs.system}.default
    inputs.ccusage.packages.${pkgs.system}.default
    zedPreview
    hermes
  ];
}
