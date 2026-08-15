{ ... }:
{
  security.polkit.enable = true;
  security.rtkit.enable = true;
  security.pam.services.hyprlock = { };

  services.gnome.gnome-keyring.enable = true;
}
