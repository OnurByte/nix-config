{ pkgs, ... }:
{
  xdg.configFile."hypr/hyprland.conf".text = ''
    monitor = ,preferred,auto,1

    exec-once = caelestia shell -d
    exec-once = nm-applet --indicator
    exec-once = wl-paste --type text --watch cliphist store
    exec-once = wl-paste --type image --watch cliphist store
    exec-once = hypridle

    input {
      kb_layout = tr,us
      kb_options = grp:caps_toggle
      follow_mouse = 1

      touchpad {
        natural_scroll = true
      }
    }

    general {
      gaps_in = 5
      gaps_out = 10
      border_size = 2
      col.active_border = rgba(33ccffee) rgba(00ff99ee) 45deg
      col.inactive_border = rgba(595959aa)
      layout = dwindle
    }

    decoration {
      rounding = 10

      blur {
        enabled = true
        size = 3
        passes = 1
      }
    }

    animations {
      enabled = true
      bezier = work, 0.05, 0.9, 0.1, 1.05
      animation = windows, 1, 6, work
      animation = windowsOut, 1, 6, default
      animation = fade, 1, 6, default
      animation = workspaces, 1, 5, work
    }

    $mainMod = SUPER
    $terminal = ${pkgs.ghostty}/bin/ghostty
    $browser = helium
    $fileManager = ${pkgs.thunar}/bin/thunar

    bind = $mainMod, RETURN, exec, $terminal
    bind = $mainMod, B, exec, $browser
    bind = $mainMod, E, exec, $fileManager
    bind = $mainMod, N, exec, $terminal -e nvim
    bind = $mainMod, SPACE, exec, caelestia shell drawers toggle launcher

    bind = $mainMod, Q, killactive,
    bind = $mainMod SHIFT, Q, exit,
    bind = $mainMod, F, fullscreen, 0
    bind = $mainMod, V, togglefloating,
    bind = $mainMod, L, exec, ${pkgs.hyprlock}/bin/hyprlock

    bindm = $mainMod, mouse:272, movewindow
    bindm = $mainMod, mouse:273, resizewindow

    bind = $mainMod, left, movefocus, l
    bind = $mainMod, right, movefocus, r
    bind = $mainMod, up, movefocus, u
    bind = $mainMod, down, movefocus, d

    bind = $mainMod, 1, workspace, 1
    bind = $mainMod, 2, workspace, 2
    bind = $mainMod, 3, workspace, 3
    bind = $mainMod, 4, workspace, 4
    bind = $mainMod, 5, workspace, 5
    bind = $mainMod, 6, workspace, 6
    bind = $mainMod, 7, workspace, 7
    bind = $mainMod, 8, workspace, 8
    bind = $mainMod, 9, workspace, 9
    bind = $mainMod, 0, workspace, 10

    bind = $mainMod SHIFT, 1, movetoworkspace, 1
    bind = $mainMod SHIFT, 2, movetoworkspace, 2
    bind = $mainMod SHIFT, 3, movetoworkspace, 3
    bind = $mainMod SHIFT, 4, movetoworkspace, 4
    bind = $mainMod SHIFT, 5, movetoworkspace, 5
    bind = $mainMod SHIFT, 6, movetoworkspace, 6
    bind = $mainMod SHIFT, 7, movetoworkspace, 7
    bind = $mainMod SHIFT, 8, movetoworkspace, 8
    bind = $mainMod SHIFT, 9, movetoworkspace, 9
    bind = $mainMod SHIFT, 0, movetoworkspace, 10

    bind = , Print, exec, grim -g "$(slurp)" - | swappy -f -
    bind = SHIFT, Print, exec, grim - | swappy -f -
    bind = $mainMod, Print, exec, grim -g "$(slurp)" - | wl-copy
    bind = $mainMod SHIFT, V, exec, cliphist list | fuzzel --dmenu | cliphist decode | wl-copy

    bind = , XF86MonBrightnessDown, exec, brightnessctl set 5%-
    bind = , XF86MonBrightnessUp, exec, brightnessctl set +5%
    bind = , XF86AudioLowerVolume, exec, wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%-
    bind = , XF86AudioRaiseVolume, exec, wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%+
    bind = , XF86AudioMute, exec, wpctl set-mute @DEFAULT_AUDIO_SINK@ toggle
    bind = , XF86AudioMicMute, exec, wpctl set-mute @DEFAULT_AUDIO_SOURCE@ toggle
  '';

  xdg.configFile."hypr/hypridle.conf".text = ''
    general {
      lock_cmd = pidof hyprlock || hyprlock
      before_sleep_cmd = loginctl lock-session
      after_sleep_cmd = hyprctl dispatch dpms on
    }

    listener {
      timeout = 300
      on-timeout = loginctl lock-session
    }

    listener {
      timeout = 600
      on-timeout = hyprctl dispatch dpms off
      on-resume = hyprctl dispatch dpms on
    }
  '';

  xdg.configFile."hypr/hyprlock.conf".text = ''
    general {
      hide_cursor = true
      grace = 2
    }

    background {
      monitor =
      path = screenshot
      blur_passes = 3
      blur_size = 8
    }

    input-field {
      monitor =
      size = 300, 50
      outline_thickness = 2
      dots_size = 0.22
      dots_spacing = 0.18
      outer_color = rgba(33ccffee)
      inner_color = rgba(11111bdd)
      font_color = rgb(cdd6f4)
      fade_on_empty = false
      placeholder_text = <i>password</i>
      position = 0, -40
      halign = center
      valign = center
    }
  '';
}
