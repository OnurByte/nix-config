{
  inputs,
  pkgs,
  ...
}:
let
  # nixpkgs fixed mat2's FFmpeg 9 regression upstream by pinning mat2 to
  # FFmpeg 8. The top-level pkgs.mat2 is only a toPythonApplication wrapper,
  # so apply the override to the underlying Python package and wrap it again.
  # This keeps mat2's test suite enabled instead of globally disabling checks.
  mat2Fixed = pkgs.python3Packages.toPythonApplication (
    pkgs.python3Packages.mat2.override {
      ffmpeg = pkgs.ffmpeg_8;
    }
  );

  onionShareSafe = pkgs.writeShellApplication {
    name = "onionshare-safe";
    runtimeInputs = [
      pkgs.coreutils
      pkgs.file
      pkgs.findutils
      mat2Fixed
      pkgs.onionshare
    ];
    text = ''
      set -euo pipefail

      if (( $# == 0 )); then
        echo "usage: onionshare-safe FILE_OR_DIRECTORY [...]" >&2
        exit 2
      fi

      runtime_root="''${XDG_RUNTIME_DIR:-/tmp}"
      staging="$(mktemp -d "$runtime_root/vesper-onionshare.XXXXXX")"
      trap 'rm -rf -- "$staging"' EXIT INT TERM

      staged=()
      for source in "$@"; do
        if [[ ! -e "$source" ]]; then
          echo "onionshare-safe: not found: $source" >&2
          exit 2
        fi

        base="$(basename -- "$source")"
        target="$staging/$base"
        if [[ -e "$target" ]]; then
          echo "onionshare-safe: duplicate top-level name: $base" >&2
          exit 2
        fi

        # Work only on a disposable copy. Symlinks are dereferenced so a share
        # never escapes the staging tree through an external target.
        cp -aL -- "$source" "$target"
        staged+=("$target")
      done

      failures=0
      while IFS= read -r -d "" item; do
        mime="$(file --brief --mime-type -- "$item")"

        case "$mime" in
          text/*|application/json|application/x-ndjson)
            # Plain text containers do not carry a separate metadata channel.
            ;;
          *)
            if ! mat2 --inplace "$item"; then
              echo "onionshare-safe: refusing unsanitized file: $item ($mime)" >&2
              failures=$((failures + 1))
            fi
            ;;
        esac
      done < <(find "$staging" -type f -print0)

      if (( failures > 0 )); then
        echo "onionshare-safe: share aborted; no unsanitized fallback was sent" >&2
        exit 1
      fi

      # OnionShare may archive directories. Normalize filesystem timestamps on
      # the staged copy so those timestamps are not carried into the archive.
      find "$staging" -depth -exec touch -h -d '@946684800' -- {} +

      echo "onionshare-safe: sanitized staged copy ready" >&2
      "${pkgs.onionshare}/bin/onionshare-cli" "''${staged[@]}"
    '';
  };
in
{
  home.packages = with pkgs; [
    # Monero reference stack. monero-cli also provides monerod,
    # monero-wallet-cli and monero-wallet-rpc without enabling a background node.
    monero-cli
    monero-gui

    # Lightweight desktop alternative with integrated Tor support.
    feather

    # BTC <-> XMR atomic-swap desktop wallet from nixpkgs.
    eigenwallet

    # Rust alternative Monero node implementation. Installed for opt-in use;
    # never started as a background service by this Home Manager profile.
    inputs.self.packages.${pkgs.system}.cuprated

    # OnionShare GUI remains useful for receive/chat/website modes. Outbound
    # file sharing should use onionshare-safe so a disposable copy is sanitized
    # before the onion service starts.
    onionshare
    onionshare-gui
    onionShareSafe
  ] ++ [
    mat2Fixed
  ];
}
