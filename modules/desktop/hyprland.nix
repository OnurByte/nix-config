{ pkgs, ... }:
{
  programs.hyprland = {
    enable = true;
    xwayland.enable = true;
  };

  services.greetd = {
    enable = true;
    settings.default_session = {
      # Hyprland 0.53+ wants sessions launched through start-hyprland rather
      # than invoking the compositor binary directly.
      command = "${pkgs.tuigreet}/bin/tuigreet --time --remember --cmd ${pkgs.hyprland}/bin/start-hyprland";
      user = "greeter";
    };
  };

  hardware.graphics.enable = true;

  hardware.bluetooth.enable = true;

  services.pipewire = {
    enable = true;
    alsa.enable = true;
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
    # Explicit backends used by Caelestia's capture/clipboard commands.
    wl-clipboard
    cliphist
    grim
    slurp
    swappy
    fuzzel
    gpu-screen-recorder

    # Native Hyprland ecosystem helpers.
    hyprsunset
    hyprpolkitagent
  ];

  fonts.packages = with pkgs; [
    inter
    noto-fonts
    noto-fonts-cjk-sans
    noto-fonts-color-emoji
    nerd-fonts.jetbrains-mono
  ];
}
