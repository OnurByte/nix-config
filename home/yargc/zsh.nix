{ ... }:
{
  programs.zsh = {
    enable = true;
    enableCompletion = true;
    autosuggestion.enable = true;
    syntaxHighlighting.enable = true;
    historySubstringSearch.enable = true;
    autocd = true;

    shellAliases = {
      nvim = "pycho";
      zed = "zed-preview";

      # Agent cockpit / quota surfaces.
      aipanel = "codexbar-popup";
      aicockpit = "bb-app";
      aicowork = "aionui";
      hermeshud = "hermes-hud";

      # Provider quota/reset data vs local token/cost accounting.
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
      xampp-start = "sudo systemctl start httpd mysql";
      xampp-stop = "sudo systemctl stop httpd mysql";
      xampp-restart = "sudo systemctl restart httpd mysql";
      xampp-status = "systemctl status httpd mysql";
    };
  };

  programs.starship = {
    enable = true;
    enableZshIntegration = true;
    settings = {
      add_newline = false;
      format = "$directory$git_branch$git_status$character";
      character = {
        success_symbol = "[❯](bold green)";
        error_symbol = "[❯](bold red)";
      };
    };
  };
}
