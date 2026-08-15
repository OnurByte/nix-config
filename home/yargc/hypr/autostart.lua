hl.on("hyprland.start", function()
    -- Caelestia owns the desktop UX. The clipboard watchers are only the
    -- cliphist backend used by Caelestia's picker, not a second shell layer.
    hl.exec_cmd("caelestia shell -d")
    hl.exec_cmd("wl-paste --type text --watch cliphist store")
    hl.exec_cmd("wl-paste --type image --watch cliphist store")

    -- Native Hyprland ecosystem helper.
    hl.exec_cmd("hyprpolkitagent")
end)
