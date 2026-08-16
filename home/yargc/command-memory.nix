{ pkgs, ... }:
let
  vesperCommandPicker = pkgs.writeShellApplication {
    name = "vesper-command-picker";
    runtimeInputs = [
      pkgs.libnotify
      pkgs.navi
      pkgs.wl-clipboard
    ];
    text = ''
      command="$(navi --print)" || exit 0
      [[ -n "$command" ]] || exit 0

      printf '%s' "$command" | wl-copy
      notify-send "Vesper Commands" "Command copied to clipboard"
      printf '\nCopied to clipboard:\n%s\n' "$command"
    '';
  };

  vesperCommands = pkgs.writeShellApplication {
    name = "vesper-commands";
    runtimeInputs = [ pkgs.ghostty ];
    text = ''
      exec ghostty -e ${vesperCommandPicker}/bin/vesper-command-picker
    '';
  };
in
{
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
    vesperCommands
  ];

  xdg.dataFile."navi/cheats/vesper.cheat".text = ''
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

    # Start Grok Build
    grok

    # Show the Nix-managed Grok Build version
    grok --version

    # Start Hermes agent
    hermes

    # Start the bb multi-agent control plane
    bb-app

    % vesper, nixos

    # Test the configured Vesper NixOS generation
    nh os test

    # Build and activate the configured Vesper NixOS generation
    nh os switch

    # Inspect Vesper hardware, storage, backup and failed units
    vesper-doctor

    # Inspect locked flake inputs
    cd ~/nix-config && nix flake metadata --no-write-lock-file

    # Update flake inputs intentionally
    cd ~/nix-config && nix flake update

    # Clean old Nix generations while keeping recent ones
    nh clean all --keep 5

    % vesper, recovery

    # Show Btrfs filesystems and scrub state
    sudo btrfs filesystem show && sudo btrfs scrub status /

    # Show Snapper snapshots for the root filesystem
    sudo snapper -c root list

    # Run the configured Restic backup now
    backup

    # Inspect the last/current Restic backup service
    backup-status

    # Verify the Restic repository
    backup-check

    # Show backup and repository-check timers
    systemctl list-timers 'vesper-backup*'

    % vesper, web

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

  xdg.desktopEntries.vesper-commands = {
    name = "Vesper Commands";
    genericName = "Command Palette";
    comment = "Search curated commands and copy one to the clipboard";
    exec = "${vesperCommands}/bin/vesper-commands";
    icon = "utilities-terminal";
    terminal = false;
    categories = [
      "Utility"
      "Development"
    ];
  };
}
