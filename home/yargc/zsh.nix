{ lib, ... }:
{
  programs.zsh = {
    enable = true;
    enableCompletion = true;
    autosuggestion.enable = true;
    syntaxHighlighting.enable = true;
    historySubstringSearch.enable = true;
    autocd = true;

    # Keep the useful part of the old Oh My Zsh philosophy without letting it
    # own the prompt or turn the shell into a giant plugin bundle.
    oh-my-zsh = {
      enable = true;
      theme = "";
      plugins = [
        "git"
        "sudo"
        "extract"
        "colored-man-pages"
      ];
    };

    shellAliases = {
      nvim = "pycho";
      zed = "zed-preview";
      glm = "zcode";

      aipanel = "codexbar-popup";
      aicockpit = "bb-app";
      hermeshud = "hermes-hud";
      aiusage = "codexbar cards";
      ailimits = "codexbar --provider all";
      aicost = "ccusage daily";
      aiweek = "ccusage weekly";
      claude-blocks = "ccusage blocks";

      ll = "eza -lah --group-directories-first";
      cat = "bat";
      rebuild = "nh os switch";
      testnix = "nh os test";
      nixup = "cd ~/nix-config && nix flake update && nh os switch";
      nixclean = "nh clean all --keep 5";

      # Native Apache + MariaDB development stack; these are not XAMPP.
      web-start = "sudo systemctl start httpd mysql";
      web-stop = "sudo systemctl stop httpd mysql";
      web-restart = "sudo systemctl restart httpd mysql";
      web-status = "systemctl status httpd mysql";
    };

    # Caelestia pushes the active Material palette to existing PTYs and stores
    # the same escape sequence for newly opened Ghostty shells. Bypass the
    # user-facing `cat = bat` alias so raw terminal escape sequences stay raw.
    initContent = lib.mkOrder 1200 ''
      if [[ -r "$HOME/.local/state/caelestia/sequences.txt" ]]; then
        command cat "$HOME/.local/state/caelestia/sequences.txt"
      fi
    '';
  };

  programs.starship = {
    enable = true;
    enableZshIntegration = true;
    settings = {
      add_newline = false;
      command_timeout = 1000;
      format = "$directory$git_branch$git_status$nix_shell$character";
      right_format = "$cmd_duration";
      directory = {
        truncation_length = 3;
        fish_style_pwd_dir_length = 1;
      };
      git_branch.symbol = "git:";
      character = {
        success_symbol = "[❯](bold green)";
        error_symbol = "[❯](bold red)";
      };
    };
  };
}
