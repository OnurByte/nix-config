{ ... }:
{
  programs.git = {
    enable = true;

    settings = {
      user = {
        name = "Onur";
        email = "51912619+OnurByte@users.noreply.github.com";
      };

      init.defaultBranch = "main";
      core.editor = "pycho";
      pull.rebase = true;
      fetch.prune = true;
      rerere.enabled = true;
      push.autoSetupRemote = true;
    };
  };
}
