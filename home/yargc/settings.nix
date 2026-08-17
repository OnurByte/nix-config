{ config, ... }:
{
  # Caelestia owns the Settings UI inside the shell process. This desktop entry
  # gives the launcher a stable application identity without spawning a second
  # settings implementation.
  home.file.".local/share/applications/vesper-settings.desktop".text = ''
    [Desktop Entry]
    Type=Application
    Version=1.0
    Name=Settings
    GenericName=System Settings
    Comment=Configure Vesper
    Icon=preferences-system
    Exec=${config.programs.caelestia.package}/bin/caelestia-shell ipc call settings open
    Terminal=false
    Categories=Settings;System;
    Keywords=settings;system;network;apps;wellbeing;AI;
    StartupNotify=false
  '';
}
