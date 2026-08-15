{ pkgs, ... }:
let
  krakenCommandPicker = pkgs.writeShellApplication {
    name = "kraken-command-picker";
    runtimeInputs = [
      pkgs.libnotify
      pkgs.navi
      pkgs.wl-clipboard
    ];
    text = ''
      command="$(navi --print)" || exit 0
      [[ -n "$command" ]] || exit 0

      printf '%s' "$command" | wl-copy
      notify-send "Kraken Commands" "Command copied to clipboard"
      printf '\nCopied to clipboard:\n%s\n' "$command"
    '';
  };

  krakenCommands = pkgs.writeShellApplication {
    name = "kraken-commands";
    runtimeInputs = [ pkgs.ghostty ];
    text = ''
      exec ghostty -e ${krakenCommandPicker}/bin/kraken-command-picker
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
  # or README snippets. The Zsh Navi widget inserts a selection for editing;
  # the desktop palette copies it so a launcher cannot execute it accidentally.
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

    # Group Codex + Claude usage by project
    turnlens report project

    # Report usage without refreshing pricing over the network
    turnlens report --offline

    # Broad daily agent usage accounting with ccusage
    ccusage daily

    # Broad weekly agent usage accounting with ccusage
    ccusage weekly

    # Show current provider quota/reset state
    codexbar cards

    % agents, coding

    # Start Codex CLI
    codex

    # Start Claude Code
    claude

    # Start OpenCode
    opencode

    # Start Hermes agent
    hermes

    # Start the bb multi-agent control plane
    bb-app

    % kraken, nixos

    # Test the configured Kraken NixOS generation
    nh os test

    # Build and activate the configured Kraken NixOS generation
    nh os switch

    # Inspect locked flake inputs
    cd ~/nix-config && nix flake metadata --no-write-lock-file

    # Update flake inputs intentionally
    cd ~/nix-config && nix flake update

    # Clean old Nix generations while keeping recent ones
    nh clean all --keep 5

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
    comment = "Search curated commands and copy one to the clipboard";
    exec = "${krakenCommands}/bin/kraken-commands";
    icon = "utilities-terminal";
    terminal = false;
    categories = [
      "Utility"
      "Development"
    ];
  };
}
