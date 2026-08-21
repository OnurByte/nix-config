hl.on("hyprland.start", function()
    hl.exec_cmd("systemctl --user import-environment DISPLAY WAYLAND_DISPLAY XDG_CURRENT_DESKTOP XDG_SESSION_TYPE && systemctl --user start hyprland-session.target")

    -- Preserve later user choices; only seed the wallpaper on a fresh profile.
    hl.exec_cmd("sh -lc 'test -s \"$HOME/.local/state/caelestia/wallpaper/path.txt\" || caelestia wallpaper -f \"$HOME/Pictures/Wallpapers/vesper-nix-dracula.png\"'")
end)

hl.on("hyprland.shutdown", function()
    os.execute("systemctl --user stop hyprland-session.target")
end)
