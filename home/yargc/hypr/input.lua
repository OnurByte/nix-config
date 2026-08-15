hl.config({
    input = {
        -- Turkish Q is the default, not a restriction.
        kb_layout = "tr,us",
        follow_mouse = 1,
        touchpad = {
            natural_scroll = true,
        },
    },
})

hl.gesture({
    fingers = 3,
    direction = "horizontal",
    action = "workspace",
})
