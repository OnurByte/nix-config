hl.on("hyprland.start", function()
    hl.exec_cmd("caelestia shell -d")
    hl.exec_cmd("wl-paste --type text --watch cliphist store")
    hl.exec_cmd("wl-paste --type image --watch cliphist store")
    hl.exec_cmd("hyprpolkitagent")
    hl.exec_cmd("vesper-control wellbeing-daemon")

    -- Preserve later user choices; only seed the wallpaper on a fresh profile.
    hl.exec_cmd("sh -lc 'test -s \"$HOME/.local/state/caelestia/wallpaper/path.txt\" || caelestia wallpaper -f \"$HOME/Pictures/Wallpapers/vesper-nix-dracula.png\"'")
end)
