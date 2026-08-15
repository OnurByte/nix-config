{ pkgs, ... }:
{
  programs.hyprland = {
    enable = true;
    xwayland.enable = true;
  };

  services.greetd = {
    enable = true;
    settings.default_session = {
      command = "${pkgs.tuigreet}/bin/tuigreet --time --remember --cmd ${pkgs.hyprland}/bin/Hyprland";
      user = "greeter";
    };
  };

  hardware.graphics = {
    enable = true;
    enable32Bit = true;
  };

  hardware.bluetooth.enable = true;
  services.blueman.enable = true;

  services.pipewire = {
    enable = true;
    alsa.enable = true;
    alsa.support32Bit = true;
    pulse.enable = true;
    wireplumber.enable = true;
  };

  services.upower.enable = true;
  services.power-profiles-daemon.enable = true;

  programs.localsend = {
    enable = true;
    openFirewall = true;
  };

  environment.sessionVariables = {
    NIXOS_OZONE_WL = "1";
    QT_QPA_PLATFORM = "wayland;xcb";
    GDK_BACKEND = "wayland,x11";
    SDL_VIDEODRIVER = "wayland,x11";
    MOZ_ENABLE_WAYLAND = "1";
    _JAVA_AWT_WM_NONREPARENTING = "1";
  };

  environment.systemPackages = with pkgs; [
    brightnessctl
    playerctl
    wl-clipboard
    grim
    slurp
    swappy
    cliphist
    fuzzel
    hyprlock
    hypridle
    hyprpicker
    networkmanagerapplet
    pavucontrol
  ];

  fonts.packages = with pkgs; [
    inter
    noto-fonts
    noto-fonts-cjk-sans
    noto-fonts-color-emoji
    nerd-fonts.jetbrains-mono
  ];
}
