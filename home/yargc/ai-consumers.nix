{ pkgs, ... }:
let
  vesperControl = pkgs.callPackage ./packages/vesper-control.nix { };
  rawOpenCode = pkgs.opencode;

  vesperOpenCode = pkgs.writeShellApplication {
    name = "opencode";
    runtimeInputs = [
      pkgs.coreutils
      vesperControl
    ];
    text = ''
      credential="$(${vesperControl}/bin/vesper-control consumer credential opencode)"
      if [ "$credential" = "native" ]; then
        exec ${rawOpenCode}/bin/opencode "$@"
      fi
      exec ${vesperControl}/bin/vesper-control credential exec "$credential" -- \
        ${rawOpenCode}/bin/opencode "$@"
    '';
  };
in
{
  # Keep Home Manager's OpenCode settings/MCP integration, replacing only the
  # executable with a Vesper credential adapter. `native` remains the default,
  # so existing OpenCode auth behavior is unchanged until a credential alias is
  # explicitly selected.
  programs.opencode.package = vesperOpenCode;
}
