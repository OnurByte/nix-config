{ inputs, ... }:
{
  imports = [
    inputs.caelestia-shell.homeManagerModules.default
  ];

  programs.caelestia = {
    enable = true;

    # Hyprland starts the shell directly. Keeping systemd off avoids coupling
    # the first migration to UWSM/session-target details.
    systemd.enable = false;

    cli = {
      enable = true;
      settings.theme.enableGtk = true;
    };
  };
}
