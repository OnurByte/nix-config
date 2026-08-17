{ config, lib, ... }:
let
  safe = value: lib.replaceStrings [ "\t" "\n" "\r" ] [ " " " " " " ] (toString value);
  mcpRows = lib.mapAttrsToList (
    name: server:
    let
      command = server.command or "";
      args = server.args or [ ];
    in
    "${safe name}\t${safe command}\t${safe (lib.concatStringsSep " " (map toString args))}"
  ) config.programs.mcp.servers;
in
{
  home.file.".config/vesper/mcp-registry.tsv".text = lib.concatStringsSep "\n" mcpRows + "\n";
}
