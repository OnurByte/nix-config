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
      pycho = "nvim";
      ll = "eza -lah --group-directories-first";
      cat = "bat";
      rebuild = "nh os switch";
      testnix = "nh os test";
      nixup = "cd ~/nix-config && nix flake update && nh os switch";
      nixclean = "nh clean all --keep 5";
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
