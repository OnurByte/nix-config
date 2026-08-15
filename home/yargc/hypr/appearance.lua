local home = os.getenv("HOME")
local schemePath = home .. "/.config/hypr/scheme/current.lua"

local fallback = {
    primary = "33ccff",
    secondary = "00ff99",
    surface = "11111b",
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
local secondary = colour("secondary", fallback.secondary)
local surface = colour("surface", fallback.surface)

hl.monitor({
    output = "",
    mode = "preferred",
    position = "auto",
    scale = 1,
})

hl.config({
    general = {
        gaps_in = 5,
        gaps_out = 10,
        border_size = 2,
        col = {
            active_border = {
                colors = { "rgba(" .. primary .. "ee)", "rgba(" .. secondary .. "ee)" },
                angle = 45,
            },
            inactive_border = "rgba(" .. surface .. "aa)",
        },
        layout = "dwindle",
        resize_on_border = true,
    },
    decoration = {
        rounding = 10,
        blur = {
            enabled = true,
            size = 3,
            passes = 1,
            new_optimizations = true,
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

hl.curve("work", {
    type = "bezier",
    points = { { 0.05, 0.9 }, { 0.1, 1.05 } },
})

hl.animation({ leaf = "windows", enabled = true, speed = 6, bezier = "work" })
hl.animation({ leaf = "windowsOut", enabled = true, speed = 6, bezier = "default" })
hl.animation({ leaf = "fade", enabled = true, speed = 6, bezier = "default" })
hl.animation({ leaf = "workspaces", enabled = true, speed = 5, bezier = "work" })
