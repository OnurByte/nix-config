local home = os.getenv("HOME")
local schemePath = home .. "/.config/hypr/scheme/current.lua"

local fallback = {
    primary = "89b4fa",
    surface = "111318",
}

local ok, scheme = pcall(dofile, schemePath)
if not ok or type(scheme) ~= "table" then
    scheme = fallback
end

local function colour(name, default)
    local value = scheme[name] or default
    return tostring(value):gsub("^#", "")
end

local primary = colour("primary", fallback.primary)
local surface = colour("surface", fallback.surface)

hl.monitor({
    output = "",
    mode = "preferred",
    position = "auto",
    scale = 1,
})

hl.config({
    general = {
        gaps_in = 8,
        gaps_out = 14,
        border_size = 1,
        col = {
            -- Keep the frame quiet and luminous rather than using a neon
            -- multi-colour gradient. The shell itself carries the glass depth.
            active_border = "rgba(" .. primary .. "66)",
            inactive_border = "rgba(" .. surface .. "4d)",
        },
        layout = "dwindle",
        resize_on_border = true,
    },
    decoration = {
        rounding = 22,
        blur = {
            enabled = true,
            size = 12,
            passes = 4,
            new_optimizations = true,
            ignore_opacity = true,
        },
        shadow = {
            enabled = true,
            range = 24,
            render_power = 2,
            color = "rgba(00000040)",
        },
    },
    animations = {
        enabled = true,
    },
    misc = {
        disable_hyprland_logo = true,
        force_default_wallpaper = -1,
    },
    dwindle = {
        preserve_split = true,
    },
})

-- Hypruse tags every window it launches as `hypruse-owned`. Keep that
-- automation boundary visible even when its runtime-injected rule does not
-- render on a particular Hyprland build/configuration.
hl.window_rule({
    name = "vesper-hypruse-owned",
    match = { tag = "hypruse-owned" },
    border_size = 2,
    border_color = "rgba(ff5555ee)",
})

hl.curve("vesper", {
    type = "bezier",
    points = { { 0.16, 1.0 }, { 0.3, 1.0 } },
})

hl.animation({ leaf = "windows", enabled = true, speed = 5, bezier = "vesper" })
hl.animation({ leaf = "windowsOut", enabled = true, speed = 5, bezier = "default" })
hl.animation({ leaf = "fade", enabled = true, speed = 5, bezier = "default" })
hl.animation({ leaf = "workspaces", enabled = true, speed = 5, bezier = "vesper" })
