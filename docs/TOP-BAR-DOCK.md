# top bar + dock

Status: proposed shell layout spec

This document defines the target Vesper desktop shell layout: a thin system bar at the top and a centred application dock at the bottom.

The goal is the macOS/GNOME separation of concerns, not a pixel clone of either desktop. The implementation stays inside Caelestia/Quickshell and follows the existing Vesper glass, palette and adaptive-icon contracts.

## research basis

The useful prior art is already close to Vesper's stack.

- `dim-ghub/midnight-shell` is a Caelestia fork that added horizontal bar positions, start/centre/end bar sections and a native app dock. Its dock uses `DesktopEntries` plus Hyprland to merge pinned and running applications, supports drag reorder, focus-or-launch, context popouts and running/focused indicators. It also derives its surfaces and states from Caelestia colour tokens instead of owning a second theme system.
- `nick-friedrich/hyprland-dock` is a standalone Quickshell/Hyprland dock with macOS-style pointer-distance magnification, focus-or-launch behaviour, running indicators and multi-monitor layer-shell handling. Its own roadmap still lists theme integration, auto-hide, reorder and context-menu work, so it is useful as an interaction reference rather than as a component to import.
- `ekremx25/quickshell` keeps dock presentation and dock/application modelling separate. Its backend merges pinned launchers with active clients and resolves focus/launch behaviour centrally. That split is a good fit for Vesper because application identity and icon identity must remain shared across launcher, dock and Apps settings.
- Noctalia and other current Quickshell shells reinforce the same general direction: bar, dock and palette-aware shell surfaces can remain one native shell instead of composing Waybar, Plank and separate theme daemons.

A Reddit scan of recent Hyprland/Quickshell ricing did not turn up a stronger implementation model than the repositories above. Reddit is useful for visual references, but this spec should follow working shell code and Vesper's own constraints rather than screenshot conventions.

Do not vendor or depend on these projects. Reuse the architectural lessons in the existing pinned Caelestia package and keep the Vesper patch small enough to replace when upstream gains equivalent primitives.

## hard constraints

- Caelestia remains the only shell/bar implementation.
- Do not add Waybar, Plank, nwg-dock, Latte, GTK dock code or another dock daemon.
- The top bar and dock run in the existing Caelestia/Quickshell process.
- Home Manager remains the declarative source of shell configuration.
- Hyprland remains the compositor and runtime window source.
- Do not add polling when Hyprland, Quickshell or desktop-entry models already expose change signals.
- Do not create a second palette, accent or icon theme system.
- Do not hardcode presentation colours for normal, hover, focused, running or selected states.
- Keep custom Caelestia changes modular and build-tested.

## shell structure

Vesper should expose two distinct shell surfaces.

```text
┌─────────────────────────────────────────────────────────────┐
│ top system bar                                              │
│ system/workspaces      clock       status/privacy/AI/power │
└─────────────────────────────────────────────────────────────┘


                         application space


                 ╭──────────────────────────╮
                 │ centred application dock │
                 ╰──────────────────────────╯
```

The top bar is a system/status surface. The bottom dock is an application launch and switching surface. Do not mix the full status stack into the dock and do not turn the top bar into a second taskbar.

## top bar

The current left-side Caelestia bar should become a thin horizontal top bar rather than being rotated wholesale into the dock.

Default layout:

```text
start                              centre                              end
logo  workspaces  active window    clock     privacy  hermes  AI  tray  status  power
```

Existing Vesper components remain available:

- `systemMonitor`
- `agentCockpit`
- `privacyHud`
- `hermesBriefing`
- `aiUsage`
- tray
- network/Bluetooth/battery and other Caelestia status icons
- power/session entry

The bar must support start, centre and end sections instead of one flat list. The clock is centred by default. Low-priority telemetry must compact or move behind an overflow/popout when horizontal space is constrained. Important privacy state, network state, battery and power controls must not disappear behind telemetry.

The top bar is persistent except in true fullscreen where policy may hide it with the rest of the shell. Maximised windows should respect its exclusive zone.

## dock role

The dock contains applications only.

It combines:

```text
launcher favourites
        +
running applications
        ↓
canonical Vesper application identity
        ↓
one dock item per application
```

Pinned applications stay in their configured order. Running but unpinned applications appear after the pinned group. A pinned application that is running must never produce a duplicate item.

Reuse `launcher.favouriteApps` as the initial pinned-app source instead of creating another favourites database. If dock ordering later needs metadata beyond that list, keep the canonical pin order in one Vesper-owned config/state model and expose it back to the launcher rather than maintaining two independent lists.

## application identity

The dock must use the canonical application identity defined in `docs/ADAPTIVE-ICONS.md`.

Identity may reconcile desktop id, `StartupWMClass`, Wayland `app_id`, X11 `WM_CLASS`, Flatpak id, executable identity, Electron app id, Steam app id, Wine/Proton launcher identity and browser PWA identity.

Do not make fuzzy window-title matching the primary identity mechanism.

This invariant applies to every Vesper-owned application surface:

```text
launcher icon == dock icon == running-state icon == app-grid icon
```

The dock should consume the same identity resolver rather than grow its own permanent regex table. Small explicit compatibility aliases are acceptable as recovery data, not as the architecture.

## icon pipeline

The dock consumes the active `Vesper-Adaptive` icon result and must not implement another icon conversion path.

`docs/ADAPTIVE-ICONS.md` remains the single source of truth for adaptive icon generation and rendering. This document only defines how the dock consumes those icons.

Rules:

- prefer the canonical Vesper application id and icon in Vesper-owned surfaces
- fall back to the resolved desktop-entry icon when no adaptive result exists
- never show a missing icon because adaptive generation failed
- theme/accent changes may re-render dock icons locally without new AI work
- do not rasterise an additional private dock icon cache unless required for Quickshell performance and keyed to the canonical rendered asset

## visual contract

The dock and top bar use the existing Vesper shell language from `AGENTS.md` and `home/yargc/caelestia.nix`.

Current global shell values remain the baseline:

```text
rounding scale       1.25
spacing scale        1.05
padding scale        1.05
animation duration   0.85
transparency base    0.68
transparency layers  0.34
```

The exact rendered geometry may use component-specific token multipliers, but those values must remain derived from the shared token system.

Visual requirements:

- layered translucent glass rather than a flat opaque panel
- readable backdrop blur
- quiet neutral or palette-tinted glass
- thin luminous border where contrast needs edge definition
- soft restrained shadow
- generous continuous rounding
- no neon multicolour borders
- no opaque Material-dashboard cards inside the dock
- no hardcoded white/black glass treatment that ignores the current palette

The result may borrow macOS proportions and motion ideas, but it must still look like the rest of Vesper.

## palette and accent behaviour

All shell state colours come from semantic Caelestia/Vesper palette tokens.

At minimum the dock needs semantic roles equivalent to:

```text
surface glass
surface border
on-surface foreground
hover tonal overlay
pressed tonal overlay
focused accent
running neutral indicator
urgent semantic indicator
```

The current accent/primary colour drives focused and selected emphasis. Wallpaper/scheme changes must update the top bar and dock through the same Caelestia theme propagation path already used by the shell.

Suggested mapping when the current Caelestia palette exposes Material-style token names:

```text
dock glass             surface/container token + Vesper transparency
normal foreground      onSurface
hover                   onSurface or primary tonal overlay at low opacity
focused indicator       primary
selected/focused halo   primary at restrained opacity
running indicator       onSurfaceVariant or equivalent neutral token
urgent                  semantic error/warning token
```

Token names are implementation details. The semantic roles above are the contract.

Do not create a dock-specific accent selector. The existing Vesper accent/theme control owns it.

## dock geometry

The dock is bottom anchored and horizontally centred.

It is content-sized rather than screen-wide. Empty space must not become an invisible full-width pointer-capturing panel.

Default sizing target:

```text
icon visual size        ~46-50 px at scale 1
item hit target         >= 48 px
inner horizontal gap    shared spacing token
outer glass padding     shared medium/large padding token
corner radius           continuous/full token
bottom screen gap       small Vesper spacing token
```

Use logical sizes and output scaling correctly. Do not assume 1x rendering.

When the application count cannot fit the usable monitor width, reduce spacing/magnification first, then allow a bounded horizontal scroll/overflow strategy. Do not let the dock clip off-screen.

## magnification

Pointer-distance magnification is allowed and should be implemented inside Quickshell rather than through compositor transforms.

The effect must be calmer than the classic exaggerated macOS dock.

Default target:

```text
hovered item max scale     1.18
nearest neighbour scale    about 1.08
far items                  1.00
```

The scale curve should be continuous based on pointer distance, not a binary hover jump. Layout compensation must prevent adjacent icons from visually colliding.

Magnification must be disabled or reduced when reduced-motion is enabled.

Do not let icon magnification expand the layer-shell exclusive zone on every pointer movement.

## item states

Every dock item can independently represent:

```text
pinned
running
focused
launching
urgent
hovered
pressed
dragging
```

State priority must remain predictable. For example, urgent state may add semantic emphasis without hiding the fact that the app is running or focused.

Running state uses a small bottom indicator. Focused state uses the current accent and may widen or brighten that indicator. Multiple windows may be represented by at most a small bounded count/segment treatment; do not draw one dot for dozens of windows.

Launching state may use a local progress/spinner treatment and must clear when a matching toplevel appears or launch failure times out.

## interaction

Primary click:

- no running window: launch the desktop entry
- one running window: focus it
- multiple running windows: focus the most recently active window for that application

A second primary click on an already focused app should not implicitly minimise it. Vesper should avoid inventing a Windows-taskbar behaviour here.

Middle click launches a new instance when the desktop entry supports it.

Right click opens a native Caelestia popout/context surface with actions such as:

- New Window when supported
- listed running windows
- Pin to Dock / Unpin from Dock
- application-specific desktop actions when available
- Quit/Close only when a reliable target exists

Scrolling over an application with multiple windows may cycle its windows if this can be done without conflicting with global bar scroll actions.

## drag and reorder

Pinned items support drag reorder directly in the dock.

Requirements:

- reordering updates the canonical favourites order
- running unpinned items cannot silently become pinned just because they were dragged within the transient area
- explicit pinning may insert a running item into the pinned group
- drag visuals use the same token/motion system
- releasing outside the dock must not lose an application or corrupt the favourites list

Drag-to-unpin by throwing an icon away is optional and should not be the only unpin interaction.

## visibility

Dock visibility modes:

```text
persistent
auto-hide
smart
```

Default: `smart`.

`persistent` reserves a stable bottom exclusive zone.

`auto-hide` uses no permanent exclusive zone and reveals from a small bottom hot zone.

`smart` behaves like auto-hide when the dock would obstruct relevant window content, otherwise it may remain visible. The initial implementation may define smart as visible on an empty desktop and hidden when a non-fullscreen window intersects the dock region, then refine overlap detection later.

Fullscreen always hides the dock immediately.

Do not dynamically resize the compositor work area on every hover/reveal in auto-hide or smart mode. That causes visible window reflow. The dock should overlay in those modes.

The reveal hot zone must be narrow and must not make the entire lower screen edge consume clicks intended for applications.

## top bar visibility

The top bar and dock have independent visibility policy.

The top bar is normally persistent and reserves its height. The dock may be smart/auto-hidden. Hiding the dock must not hide status, privacy or power state from the top bar.

True fullscreen may hide both surfaces, with a deliberate edge reveal path if Vesper needs shell access while fullscreen.

## multi-monitor

The top bar may exist on every eligible monitor.

The dock supports:

```text
primary
all
focused
```

Default target: `primary` until focused-monitor behaviour is proven not to jump distractingly during ordinary window focus changes.

Pinned applications are global. Running state is derived from all matching application windows, while focus emphasis follows the active toplevel. A future per-monitor filtering mode may show only windows belonging to that monitor, but it must not fork application identity or pin state.

Monitor hotplug must not require a shell restart.

## popouts and z-order

Dock tooltips, window lists and context menus should reuse Caelestia popout primitives rather than implement independent popup windows with unrelated styling.

Popouts must:

- stay above the dock
- clamp to monitor bounds
- survive dock magnification without jumping
- close predictably when focus/pointer leaves the dock context
- use the same glass, palette, radius and shadow tokens

The dock must not steal keyboard focus merely because the pointer crosses it.

## performance

The dock is event driven.

Use existing signals/models for:

- Hyprland toplevel creation/removal/focus
- desktop entry changes
- favourite-app changes
- theme/accent changes
- monitor changes

Avoid fixed-interval processes that repeatedly call `hyprctl clients` or rescan desktop files.

Expensive icon work belongs to the adaptive-icon engine. The QML dock should only resolve and display already-available local assets.

Animations should stop when the dock is hidden and no transition is running.

## accessibility and motion

The dock must keep a usable pointer target even when an icon is visually smaller than the hit target.

Reduced-motion mode disables magnification and replaces spring/bounce effects with short opacity/position transitions or no transition.

Reduced-transparency mode raises glass opacity and may disable backdrop blur while preserving border/foreground contrast.

Keyboard navigation is desirable for a later pass but should not force a hidden dock into the tab order during ordinary application use.

## config model

The exact upstream schema may change, but Vesper should expose a declarative model equivalent to:

```nix
programs.caelestia.settings = {
  topBar = {
    enabled = true;
    position = "top";
    persistent = true;
    monitors = "all";
  };

  dock = {
    enabled = true;
    position = "bottom";
    alignment = "center";
    visibility = "smart";
    monitors = "primary";

    showRunning = true;
    groupWindows = true;
    useLauncherFavourites = true;

    iconSize = 48;
    magnification = {
      enabled = true;
      maxScale = 1.18;
      neighbourScale = 1.08;
    };
  };
};
```

This example describes Vesper's desired public configuration, not the current upstream Caelestia schema.

If extending Caelestia's C++ config layer is necessary, isolate the patch by responsibility. Do not scatter Vesper-specific dock constants across unrelated QML files.

## settings UI

Caelestia settings may expose the dock under the existing shell/panels or appearance area.

Useful controls:

- visibility: persistent / smart / auto-hide
- icon size
- magnification on/off and strength
- primary/all/focused monitor mode
- show running unpinned apps
- reset pinned order to launcher favourites

Do not expose colour pickers here. Accent and palette stay in the existing theme controls.

Advanced application identity fixes belong to Apps settings, not dock settings.

## implementation shape

Prefer a small patch set with clear ownership.

Conceptually:

```text
Caelestia shell
├── top system bar
│   ├── start section
│   ├── centre section
│   └── end section
├── VesperDock surface
│   ├── DockModel
│   ├── DockItem
│   ├── DockContextPopout
│   └── DockVisibilityController
└── shared services
    ├── Hyprland state
    ├── DesktopEntries/AppDB
    ├── Vesper canonical app identity
    ├── Colours/Tokens
    └── Vesper adaptive icons
```

The exact filenames can follow upstream Caelestia conventions. The important boundary is that application modelling is not embedded ad hoc inside each visual delegate.

## implementation order

1. Add horizontal top-bar layout with start/centre/end sections and migrate the current Vesper entries without losing functionality.
2. Add a separate bottom-centred dock surface in the same Quickshell process.
3. Build the dock model from launcher favourites plus running Hyprland toplevels using canonical app identity.
4. Connect the dock to `Vesper-Adaptive` icons and shared palette/tokens.
5. Add running/focused/launching states and native popouts.
6. Add drag reorder and pin/unpin.
7. Add restrained magnification and reduced-motion behaviour.
8. Add persistent/auto-hide/smart visibility without dynamic work-area jitter.
9. Add multi-monitor policy and hotplug handling.
10. Add settings controls only after the underlying config is declarative and stable.

## validation

A change is not complete until these behaviours are verified.

```text
[ ] only one shell implementation is running
[ ] no Waybar/Plank/extra dock daemon was added
[ ] top bar is horizontal and retains Vesper status/AI/privacy surfaces
[ ] dock is bottom-centred and content-sized
[ ] pinned + running instances deduplicate by canonical application identity
[ ] Flatpak, Electron, Steam and browser-PWA cases do not collapse into obvious wrong identities
[ ] adaptive icon == launcher icon == dock icon for known applications
[ ] broken adaptive icon falls back safely
[ ] accent change updates dock/top-bar focused states
[ ] light/dark or wallpaper palette change updates glass and foregrounds coherently
[ ] no normal visual state depends on hardcoded RGB/hex values
[ ] magnification does not alter the compositor exclusive zone
[ ] reduced-motion disables magnification/bounce
[ ] fullscreen hides the dock
[ ] smart/auto-hide reveal does not reflow application windows
[ ] dock does not capture pointer input across unused screen width
[ ] pinned reorder persists declaratively
[ ] right-click popout clamps to the monitor
[ ] monitor hotplug does not require a shell restart
[ ] no periodic hyprctl/desktop-entry polling was introduced
[ ] configured Caelestia package builds
[ ] full Home Manager/NixOS evaluation still succeeds
```

## non-goals

- exact macOS pixel reproduction
- GNOME Shell or Dash-to-Dock dependency
- a second launcher
- a second application identity database
- a second icon theme/render pipeline
- a dock-specific colour theme
- GTK-based shell surfaces
- moving system telemetry, privacy state or power controls into the dock

## upstream migration

The pinned Caelestia version currently assumes a vertical bar and does not expose a bar position/orientation property. Other Caelestia-derived work demonstrates that horizontal positions, sectioned layouts and a native dock can be added without abandoning the shell architecture.

Keep Vesper's implementation narrow enough that future upstream Caelestia support can replace individual pieces. Prefer adapting upstream primitives over maintaining a permanent fork when equivalent behaviour lands upstream.
