{
  inputs,
  lib,
  pkgs,
  username,
  ...
}:
let
  flakeInputs = lib.filterAttrs (_: lib.isType "flake") inputs;
in
{
  nixpkgs.config.allowUnfree = true;

  nix = {
    settings = {
      experimental-features = [
        "nix-command"
        "flakes"
      ];
      auto-optimise-store = true;
      trusted-users = [
        "root"
      ];

      # numtide/llm-agents.nix publishes CI-built agent packages here. Without
      # the cache, large Electron/Rust agent surfaces would rebuild locally.
      extra-substituters = [ "https://cache.numtide.com" ];
      extra-trusted-public-keys = [
        "niks3.numtide.com-1:DTx8wZduET09hRmMtKdQDxNNthLQETkc/yaX7M4qK0g="
      ];
    };

    channel.enable = false;
    registry = lib.mapAttrs (_: flake: { inherit flake; }) flakeInputs;
    nixPath = lib.mapAttrsToList (name: _: "${name}=flake:${name}") flakeInputs;
  };

  programs.nh = {
    enable = true;
    clean.enable = true;
    clean.extraArgs = "--keep 5 --keep-since 7d";
    flake = "/home/${username}/nix-config";
  };

  programs.nix-ld.enable = true;

  environment.systemPackages = with pkgs; [
    git
    nix-output-monitor
  ];
}
