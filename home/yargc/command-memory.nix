{ pkgs, ... }:
let
  krakenCommands = pkgs.writeShellApplication {
    name = "kraken-commands";
    runtimeInputs = [
      pkgs.ghostty
      pkgs.navi
    ];
    text = ''
      exec ghostty -e navi
    '';
  };
in
{
  # Ctrl-R becomes a contextual local command-memory database. Keep it offline:
  # no account, sync or update checks are needed on this single-user workstation.
  programs.atuin = {
    enable = true;
    enableZshIntegration = true;
    flags = [ "--disable-up-arrow" ];
    forceOverwriteSettings = true;
    settings = {
      auto_sync = false;
      update_check = false;
      search_mode = "fuzzy";
      filter_mode = "global";
      workspaces = true;
      style = "compact";
      inline_height = 20;
      show_preview = true;
      enter_accept = false;
    };
  };

  home.packages = [
    pkgs.navi
    krakenCommands
  ];

  # Curated commands live in one searchable source instead of scattered aliases
  # or README snippets. Navi can insert a selection for editing before execution.
  xdg.dataFile."navi/cheats/kraken.cheat".text = ''
    % agents, usage, turnlens

    # Watch a Codex session turn-by-turn
    turnlens codex

    # Watch a Claude Code session turn-by-turn
    turnlens claude

    # Daily Codex + Claude usage report
    turnlens report

    # Weekly Codex + Claude usage report
    turnlens report weekly

    # Group agent usage by project
    turnlens report project

    # Run TurnLens without refreshing pricing over the network
    turnlens report --offline

    % agents, coding

    # Start Codex CLI
    codex

    # Start Claude Code
    claude

    # Start OpenCode
    opencode

    # Start Hermes agent
    hermes

    % kraken, nixos

    # Evaluate Kraken without activating it
    sudo nixos-rebuild test --flake .#kraken

    # Activate the current Kraken flake
    sudo nixos-rebuild switch --flake .#kraken

    # Inspect locked flake inputs
    nix flake metadata --no-write-lock-file

    # Update flake inputs intentionally
    nix flake update

    % kraken, web

    # Start local Apache + PHP + MariaDB development stack
    web-start

    # Stop local web development stack
    web-stop

    # Restart local web development stack
    web-restart

    # Show local web development stack status
    web-status

    % git, github

    # Show Git working tree state
    git status

    # Show GitHub CLI authentication state
    gh auth status

    # Open the current GitHub repository in the browser
    gh repo view --web

    % privacy, monero

    # Show Cuprate node options without starting a node
    cuprated --help

    # Show Monero daemon options without starting a node
    monerod --help

    # Show Monero wallet CLI options
    monero-wallet-cli --help

    % shell, memory

    # Search rich shell history interactively
    atuin search -i
  '';

  xdg.desktopEntries.kraken-commands = {
    name = "Kraken Commands";
    genericName = "Command Palette";
    comment = "Search remembered commands and insert one for editing";
    exec = "${krakenCommands}/bin/kraken-commands";
    icon = "utilities-terminal";
    terminal = false;
    categories = [
      "Utility"
      "Development"
    ];
  };
}
