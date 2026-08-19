local main = "SUPER"
local locked = { locked = true }
local repeating = { repeating = true }
local lockedRepeating = { locked = true, repeating = true }

local function exec(command)
    return hl.dsp.exec_cmd(command)
end

hl.bind(main .. " + Return", exec("ghostty +new-window"))
hl.bind(main .. " + B", exec("zen-beta"))
hl.bind(main .. " + SHIFT + B", exec("helium"))
hl.bind(main .. " + E", exec("thunar"))
hl.bind(main .. " + N", exec("ghostty -e pycho"))
hl.bind(main .. " + Z", exec("zeditor"))
hl.bind(main .. " + Space", exec("vicinae toggle"))
hl.bind(main .. " + C", hl.dsp.global("caelestia:showall"))
hl.bind(main .. " + SHIFT + N", hl.dsp.global("caelestia:sidebar"))
hl.bind(main .. " + L", hl.dsp.global("caelestia:lock"))
hl.bind(main .. " + SHIFT + Q", hl.dsp.global("caelestia:session"))
hl.bind(main .. " + SHIFT + Space", exec("hyprctl switchxkblayout all next"))
hl.bind(main .. " + slash", exec("vesper-commands"))

hl.bind(main .. " + M", exec("spotify"))
hl.bind(main .. " + D", exec("vesktop"))
hl.bind(main .. " + A", exec("chatgpt"))
hl.bind(main .. " + SHIFT + A", exec("claude-desktop"))

hl.bind(main .. " + G", exec("ghostty -e grok"))
hl.bind(main .. " + SHIFT + H", exec("hermes-desktop"))
hl.bind(main .. " + T", exec("t3code-nightly"))
hl.bind(main .. " + U", exec("codexbar-popup"))
hl.bind(main .. " + SHIFT + C", exec("ghostty -e codex"))
hl.bind(main .. " + SHIFT + O", exec("ghostty -e opencode"))

hl.bind(main .. " + SHIFT + BackSpace", exec("vesper-hypruse-mcp stop"))

hl.bind(main .. " + Q", hl.dsp.window.close())
hl.bind(main .. " + F", hl.dsp.window.fullscreen({ mode = "fullscreen" }))
hl.bind(main .. " + V", hl.dsp.window.float({ action = "toggle" }))
hl.bind("ALT + Tab", hl.dsp.window.cycle_next(), repeating)
hl.bind("ALT + SHIFT + Tab", hl.dsp.window.cycle_next({ next = false }), repeating)

for _, direction in ipairs({ "left", "right", "up", "down" }) do
    hl.bind(main .. " + " .. direction, hl.dsp.focus({ direction = direction }))
    hl.bind(main .. " + SHIFT + " .. direction, hl.dsp.window.move({ direction = direction }))
end

hl.bind(main .. " + mouse:272", hl.dsp.window.drag(), { mouse = true })
hl.bind(main .. " + mouse:273", hl.dsp.window.resize(), { mouse = true })

for i = 1, 10 do
    local key = i % 10
    hl.bind(main .. " + " .. key, hl.dsp.focus({ workspace = i }))
    hl.bind(main .. " + SHIFT + " .. key, hl.dsp.window.move({ workspace = i }))
end
hl.bind(main .. " + mouse_down", hl.dsp.focus({ workspace = "e+1" }))
hl.bind(main .. " + mouse_up", hl.dsp.focus({ workspace = "e-1" }))

hl.bind("Print", exec("caelestia screenshot"), locked)
hl.bind("SHIFT + Print", exec("caelestia screenshot -r -f"), locked)
hl.bind(main .. " + Print", exec("caelestia screenshot -r"), locked)
hl.bind(main .. " + CTRL + O", exec("vesper-ocr"))
hl.bind(main .. " + SHIFT + R", exec("caelestia record -r"))
hl.bind(main .. " + CTRL + R", exec("caelestia record -r -s"))
hl.bind(main .. " + SHIFT + V", exec("pkill fuzzel || caelestia clipboard"))
hl.bind(main .. " + period", exec("pkill fuzzel || caelestia emoji -p"))

hl.bind("XF86MonBrightnessUp", hl.dsp.global("caelestia:brightnessUp"), lockedRepeating)
hl.bind("XF86MonBrightnessDown", hl.dsp.global("caelestia:brightnessDown"), lockedRepeating)
hl.bind("XF86AudioPlay", hl.dsp.global("caelestia:mediaToggle"), locked)
hl.bind("XF86AudioPause", hl.dsp.global("caelestia:mediaToggle"), locked)
hl.bind("XF86AudioNext", hl.dsp.global("caelestia:mediaNext"), locked)
hl.bind("XF86AudioPrev", hl.dsp.global("caelestia:mediaPrev"), locked)
hl.bind("XF86AudioRaiseVolume", exec("wpctl set-mute @DEFAULT_AUDIO_SINK@ 0; wpctl set-volume -l 1 @DEFAULT_AUDIO_SINK@ 5%+"), lockedRepeating)
hl.bind("XF86AudioLowerVolume", exec("wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%-"), lockedRepeating)
hl.bind("XF86AudioMute", exec("wpctl set-mute @DEFAULT_AUDIO_SINK@ toggle"), locked)
hl.bind("XF86AudioMicMute", exec("wpctl set-mute @DEFAULT_AUDIO_SOURCE@ toggle"), locked)
