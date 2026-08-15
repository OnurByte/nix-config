{
  description = "Kraken — an agentic NixOS rice";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    home-manager = {
      url = "github:nix-community/home-manager";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    caelestia-shell = {
      url = "github:caelestia-dots/shell";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    helium = {
      url = "github:schembriaiden/helium-browser-nix-flake";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    zen-browser = {
      url = "github:0xc000022070/zen-browser-flake";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.home-manager.follows = "home-manager";
    };

    spicetify-nix = {
      url = "github:Gerg-L/spicetify-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    # Native vendor desktop applications packaged for NixOS.
    chatgpt-desktop = {
      url = "github:poeck/chatgpt-desktop-app-nix-flake";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    claude-desktop = {
      url = "github:heytcass/claude-desktop-linux-flake";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    # CodexBar's upstream Linux CLI plus the Wayland GTK usage surface source.
    codexbar = {
      url = "github:alioguzhan/codexbar-flake";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    codexbar-ui-src = {
      url = "github:Marouan-chak/codexbar-waybar";
      flake = false;
    };

    # Daily-updated Nix packages for coding agents, agent control planes and
    # agent-native tooling. Keep its own tested nixpkgs pin for cache hits.
    llm-agents.url = "github:numtide/llm-agents.nix";
  };

  outputs =
    {
      nixpkgs,
      home-manager,
      ...
    }@inputs:
    let
      system = "x86_64-linux";
      username = "yargc";
      hostname = "kraken";
      pkgs = nixpkgs.legacyPackages.${system};
    in
    {
      formatter.${system} = pkgs.nixfmt-rfc-style;

      packages.${system} = {
        zcode = pkgs.callPackage ./home/yargc/packages/zcode.nix { };
        cuprated = pkgs.callPackage ./home/yargc/packages/cuprated.nix { };
        turnlens = pkgs.callPackage ./home/yargc/packages/turnlens.nix { };
      };

      nixosConfigurations.${hostname} = nixpkgs.lib.nixosSystem {
        inherit system;
        specialArgs = {
          inherit inputs username hostname;
        };

        modules = [
          ./hosts/${hostname}
          home-manager.nixosModules.home-manager
          {
            home-manager = {
              useGlobalPkgs = true;
              useUserPackages = true;
              backupFileExtension = "hm-backup";
              extraSpecialArgs = {
                inherit inputs username hostname;
              };
              users.${username} = import ./home/${username};
            };
          }
        ];
      };
    };
}
