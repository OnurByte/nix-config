{
  inputs,
  pkgs,
  ...
}:
let
  codexbar = inputs.codexbar.packages.${pkgs.system}.default;
  codexbarUi = pkgs.callPackage ./packages/codexbar-ui.nix {
    src = inputs.codexbar-ui-src;
    inherit codexbar;
  };

  agenticCaelestia = inputs.caelestia-shell.packages.${pkgs.system}.with-cli.overrideAttrs (old: {
    patches = (old.patches or [ ]) ++ [ ./packages/caelestia-codexbar.patch ];

    postPatch = (old.postPatch or "") + ''
      substitute ${./packages/CodexUsage.qml} modules/bar/components/CodexUsage.qml \
        --subst-var-by codexbarStatus ${codexbarUi}/bin/codexbar-status \
        --subst-var-by codexbarPopup ${codexbarUi}/bin/codexbar-popup
    '';
  });
in
{
  imports = [
    inputs.caelestia-shell.homeManagerModules.default
  ];

  programs.caelestia = {
    enable = true;
    package = agenticCaelestia;

    # Hyprland starts the shell directly.
    systemd.enable = false;

    # The stock Caelestia list accepts arbitrary entry IDs; the patched shell
    # gives aiUsage a native QML delegate backed by CodexBar.
    settings.bar.entries = [
      { id = "logo"; enabled = true; }
      { id = "workspaces"; enabled = true; }
      { id = "spacer"; enabled = true; }
      { id = "activeWindow"; enabled = true; }
      { id = "spacer"; enabled = true; }
      { id = "tray"; enabled = true; }
      { id = "aiUsage"; enabled = true; }
      { id = "clock"; enabled = true; }
      { id = "statusIcons"; enabled = true; }
      { id = "power"; enabled = true; }
    ];

    cli = {
      enable = true;
      settings.theme.enableGtk = true;
    };
  };

  home.packages = [ codexbarUi ];

  # The popup expects provider marks under XDG_DATA_HOME. Keep them immutable
  # in the Nix store and expose that directory through Home Manager.
  xdg.dataFile."codexbar-waybar/icons".source = "${codexbarUi}/share/codexbar-waybar/icons";
}
