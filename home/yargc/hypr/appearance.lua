local home = os.getenv("HOME")
local schemePath = home .. "/.config/hypr/scheme/current.lua"

local fallback = {
    primary = "89b4fa",
    secondary = "cba6f7",
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
        gaps_in = 6,
        gaps_out = 10,
        border_size = 1,
        col = {
            active_border = {
                colors = { "rgba(" .. primary .. "cc)", "rgba(" .. secondary .. "aa)" },
                angle = 45,
            },
            inactive_border = "rgba(" .. surface .. "66)",
        },
        layout = "dwindle",
        resize_on_border = true,
    },
    decoration = {
        rounding = 16,
        blur = {
            enabled = true,
            size = 8,
            passes = 3,
            new_optimizations = true,
            ignore_opacity = true,
        },
        shadow = {
            enabled = true,
            range = 18,
            render_power = 3,
            color = "rgba(00000055)",
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

hl.curve("vesper", {
    type = "bezier",
    points = { { 0.16, 1.0 }, { 0.3, 1.0 } },
})

hl.animation({ leaf = "windows", enabled = true, speed = 6, bezier = "vesper" })
hl.animation({ leaf = "windowsOut", enabled = true, speed = 6, bezier = "default" })
hl.animation({ leaf = "fade", enabled = true, speed = 6, bezier = "default" })
hl.animation({ leaf = "workspaces", enabled = true, speed = 5, bezier = "vesper" })
