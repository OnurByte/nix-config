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
      hermes = "hermes-bootstrap";
      aiusage = "codexbar cards";
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
