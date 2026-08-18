# liquid glass top bar + dock

Status: implementation spec

This document defines Vesper's target shell layout and material behavior.

The target is a macOS-style top system bar plus a centred bottom application dock rendered with an Apple Liquid Glass material language. This is not a generic frosted-glass theme, not a visionOS-inspired approximation and not a collection of translucent Material cards.

The implementation stays inside Caelestia/Quickshell and reuses Vesper's existing palette, adaptive-icon and application-identity systems.

## mission

Replace the current left-side taskbar layout with two native Caelestia surfaces:

```text
┌──────────────────────────────────────────────────────────────┐
│ liquid glass top bar                                        │
│ system/workspaces        clock        privacy/status/power   │
└──────────────────────────────────────────────────────────────┘


                         application space


                  ╭────────────────────────╮
                  │ liquid glass app dock  │
                  ╰────────────────────────╯
```

The top bar owns system state and shell controls.

The dock owns applications only.

Both surfaces use one shared Liquid Glass renderer and one shared theme/token system. Do not implement separate glass recipes for the dock, bar, popouts and context menus.

## research basis

The useful prior art is close to Vesper's actual stack.

- `dim-ghub/midnight-shell` proves that Caelestia can support horizontal bar positions, start/centre/end sections and a native application dock without introducing another shell. Its dock combines `DesktopEntries` with Hyprland state, supports pinned/running deduplication, drag reorder, focus-or-launch, context popouts and running/focused indicators.
- `nick-friedrich/hyprland-dock` demonstrates pointer-distance dock magnification, focus-or-launch behaviour, running indicators and multi-monitor layer-shell handling in Quickshell.
- `ekremx25/quickshell` separates dock presentation from application modelling. Vesper should follow the same separation because application identity belongs to a shared service, not to dock QML.
- Noctalia and similar Quickshell shells confirm that bar, dock, popouts and palette-aware presentation can remain inside one native shell process.

These projects are architectural references only. Do not vendor them and do not add them as runtime dependencies.

## non-negotiable constraints

- Caelestia remains the only desktop shell/bar implementation.
- Do not add Waybar, Plank, nwg-dock, Latte, GTK dock code or another dock daemon.
- The top bar and dock run in the existing Caelestia/Quickshell process.
- Home Manager owns declarative configuration and installation.
- Hyprland remains the compositor and source of live window state.
- Do not poll `hyprctl clients` on an interval when Quickshell/Hyprland models already expose state changes.
- Do not create another accent, palette or icon theme system.
- Do not create a second application identity resolver inside the dock.
- Do not hardcode normal, hover, focused or running colours independently of the active Vesper palette.
- Keep the Caelestia patch modular and replaceable if upstream later grows equivalent primitives.
- Every changed Caelestia/QML path must be build-tested according to `AGENTS.md`.

## Liquid Glass is the visual contract

The shell material is Apple Liquid Glass.

Do not describe the target as:

```text
visionOS inspired
frosted acrylic
blurred translucent panel
glassmorphism
Material glass
```

Those may overlap visually but they are not the contract.

For Vesper, a Liquid Glass surface is a dynamic material composed from several effects that behave as one object:

```text
background content
      ↓
backdrop sampling
      ↓
controlled blur
      ↓
subtle refraction / optical displacement
      ↓
adaptive material tint and contrast compensation
      ↓
contour-aware edge luminance
      ↓
specular response
      ↓
soft depth shadow
      ↓
crisp foreground content
```

A plain rectangle with opacity plus blur does not satisfy this spec.

The implementation does not need Apple's private renderer or shaders. It must reproduce the public visual behaviour with Quickshell/Qt primitives and small custom shader/effect code where necessary.

## one shared Liquid Glass renderer

Create one shared Vesper/Caelestia material primitive conceptually equivalent to:

```text
LiquidGlassSurface
├── backdrop sampler
├── blur stage
├── optical/refraction stage
├── adaptive tint stage
├── contour/rim response
├── specular response
├── shadow/depth response
└── foreground slot
```

The exact QML/component names may differ.

The top bar, dock, dock popouts and related shell surfaces consume the same primitive with different geometry and material parameters.

Do not duplicate shader constants across components.

Do not turn every child control into another independent sheet of glass. Related controls should normally sit inside one coherent glass container.

## material variants

Vesper needs two semantic Liquid Glass variants:

```text
regular
clear
```

`regular` is the default shell material. It prioritises legibility while preserving visible background interaction.

`clear` is more transparent and visually lighter. Use it only where the underlying wallpaper/window content remains readable and foreground contrast can be guaranteed.

The renderer may automatically strengthen tint/blur or fall back from `clear` toward `regular` when background contrast becomes unsafe.

The user should not need to tune raw shader parameters to get readable shell text.

## backdrop blur

Blur exists to separate content planes, not to erase the wallpaper.

Requirements:

- preserve broad colour and luminance information from the content behind the surface
- avoid a milky opaque rectangle
- avoid excessive blur that makes every wallpaper converge to grey
- scale blur correctly for output scale
- keep foreground icons/text outside the blur pass
- avoid recursively sampling other Vesper glass layers when a simpler compositing path is available

Blur strength may vary between regular and clear material, but it must be controlled centrally by the material renderer.

## refraction and optical displacement

Liquid Glass must have a restrained optical response beyond blur.

Use a small displacement/refraction field tied to surface geometry. The effect should make the material feel optically thick without making text or wallpaper look warped.

Requirements:

- displacement is strongest near the glass contour and extremely small through the centre
- rounded corners affect the refraction field continuously
- the effect must remain stable while the surface animates
- do not introduce visible wobble while the pointer is stationary
- do not distort foreground icons or text
- disable or simplify this pass if required by reduced-transparency/performance mode

This should read as material thickness, not as a water-ripple filter.

## contour and specular response

A static white 1 px border is not an adequate Liquid Glass edge.

The glass contour should have luminance variation derived from shape, background and a stable virtual light direction.

Use a restrained combination of:

```text
outer edge luminance
inner edge highlight
soft specular lobe
subtle opposite-side darkening
```

Requirements:

- continuous around rounded corners
- no neon outline
- no permanently bright white ring
- no chromatic RGB border
- highlight strength adapts to light/dark wallpaper conditions
- focused/hovered state may alter material response slightly but must not replace the normal accent-state semantics

The dock should look like one optically coherent piece of shaped glass.

## adaptive tint and contrast

Liquid Glass responds to what is behind it.

The material renderer should derive enough backdrop luminance/chroma information to keep foreground content legible without destroying the background relationship.

Use the current Vesper palette as the semantic colour source, but apply it as a restrained material tint rather than painting the entire dock an opaque accent colour.

The renderer may adjust:

```text
material tint opacity
blur strength within a bounded range
foreground contrast role
rim/specular strength
shadow strength
```

Do not create a dock-specific accent picker.

Wallpaper or palette changes must update the dock and top bar through the same existing Caelestia/Vesper theme path.

## depth and shadows

Glass surfaces need depth separation from the desktop without looking like floating opaque cards.

Use:

- a broad low-opacity outer shadow
- subtle contact/depth shadow near the material edge
- optional weak inner luminance variation

Do not use heavy black drop shadows or fake 3D bevels.

Shadow geometry must follow the actual continuous rounded shape.

## motion behaviour

Liquid Glass responds as a material during geometry changes.

When the dock grows, shrinks, reveals, hides or changes item count:

- container geometry interpolates continuously
- corner shape remains continuous
- backdrop/material effects track the animated geometry
- specular/rim response does not pop between static textures
- foreground content remains crisp

Do not animate by cross-fading between pre-rendered glass screenshots.

For ordinary state changes, prefer smooth spatial interpolation and opacity/tint changes over bounce-heavy UI animation.

Reduced-motion mode disables magnification and spring-like behaviour while preserving instant or short material transitions.

## top bar

The current left-side Caelestia bar becomes a horizontal top system bar.

It is not the dock rotated 90 degrees.

Default structure:

```text
start                              centre                             end
logo  workspaces  active window    clock       privacy  AI  tray  status  power
```

Existing Vesper components remain available, including:

- `systemMonitor`
- `agentCockpit`
- `privacyHud`
- `hermesBriefing`
- `aiUsage`
- tray
- network/Bluetooth/battery and other Caelestia status icons
- power/session controls

The bar must expose start, centre and end sections instead of one flat vertical entry array.

Clock is centred by default.

When horizontal space is constrained, low-priority telemetry compacts or moves behind a native overflow/popout. Privacy, network, battery and power state have higher visibility priority than telemetry.

The top bar normally reserves a stable exclusive zone. Maximised windows respect it.

True fullscreen may hide the bar according to shell fullscreen policy.

## top bar Liquid Glass geometry

The bar should read as a light shell strip rather than a thick opaque taskbar.

Two acceptable geometry modes may be implemented:

```text
continuous strip
floating inset strip
```

Default target: `floating inset strip` if it remains visually stable with Caelestia drawers/popouts, otherwise use a continuous top strip.

The material is regular Liquid Glass by default.

Do not wrap every status icon in an individual glass capsule. Group related status controls into coherent hit regions while the bar itself remains the primary material surface.

## dock role

The bottom dock contains applications only.

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

Pinned applications remain in configured order.

Running unpinned applications appear after the pinned group.

A pinned application that is running produces one item, never two.

Reuse `launcher.favouriteApps` as the initial pinned source instead of creating another favourites database.

## canonical application identity

The dock consumes the canonical application identity defined by `docs/ADAPTIVE-ICONS.md`.

Identity may reconcile:

```text
desktop id
StartupWMClass
Wayland app_id
X11 WM_CLASS
Flatpak id
executable identity
Electron app id
Steam app id
Wine/Proton launcher identity
browser PWA identity
explicit aliases
```

Window title is not the primary identity mechanism.

The invariant is:

```text
launcher icon == dock icon == running-state icon == app-grid icon
```

The dock must not grow a permanent private regex map that diverges from Apps settings or the icon pipeline.

## adaptive icons

The dock consumes Vesper's canonical adaptive icon result.

`docs/ADAPTIVE-ICONS.md` remains the source of truth for icon decomposition, canonical identity, appearances and rendering.

Dock rules:

- prefer canonical Vesper icon identity
- fall back to the real desktop-entry icon if no accepted adaptive result exists
- never leave a blank/missing launcher because adaptive generation failed
- palette/appearance changes may re-render locally without new AI work
- do not build a second dock-specific icon conversion pipeline

Adaptive icons must visually belong inside Liquid Glass without being flattened into the dock material. Application artwork remains its own foreground object.

## dock geometry

The dock is bottom anchored, horizontally centred and content-sized.

It must not be a screen-wide invisible input panel.

Baseline target at logical scale 1:

```text
icon visual size       48 px
minimum hit target     48 px
outer glass padding    shared medium/large token
inner spacing          shared spacing token
bottom screen gap      small shared spacing token
shape                   continuous capsule/squircle-derived container
```

Use logical dimensions and real output scaling. Do not assume 1x.

The material surface smoothly resizes as items appear/disappear.

When too many applications exist for usable monitor width:

1. reduce magnification amplitude
2. reduce spacing within bounded limits
3. allow bounded horizontal overflow/scroll

Never clip dock items off-screen.

## magnification

Use pointer-distance magnification similar to macOS dock behaviour, implemented inside Quickshell.

Default target:

```text
hovered item max scale     1.18
nearest neighbour          ~1.08
far items                  1.00
```

Requirements:

- continuous distance curve instead of binary hover scaling
- layout compensation prevents icon collisions
- hit regions remain stable enough for precise pointing
- material container may expand smoothly when necessary
- magnification does not constantly rewrite the compositor exclusive zone
- reduced-motion disables it

Do not exaggerate magnification to classic novelty-dock levels.

## dock item states

Each item may represent:

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

The glass container remains coherent while foreground state treatment changes.

Running state uses a restrained bottom indicator.

Focused state uses the current semantic accent and may widen/brighten the indicator.

Multiple windows use a bounded count/segment representation rather than unlimited dots.

Launching state may show a compact local progress treatment and clears when the matching toplevel appears or launch failure expires.

Urgent state uses semantic urgency colour without hiding running/focused state.

## hover and press material response

Do not place a permanent mini glass tile behind every icon.

Normal state: icon floats cleanly inside the shared dock material.

Hover may create a very restrained local lens/highlight response around the item, derived from the same Liquid Glass renderer.

Press may briefly increase local tint/edge response or apply a small scale depression.

The interaction must read as deformation/response of one shared material rather than a stack of independent cards.

## interaction

Primary click:

- no running window -> launch desktop entry
- one running window -> focus it
- multiple running windows -> focus the most recently active window for that app

Clicking an already focused app does not implicitly minimise it.

Middle click launches a new instance when supported.

Right click opens a native Caelestia Liquid Glass context popout with actions such as:

- New Window
- running window list
- Pin to Dock / Unpin from Dock
- desktop-entry actions
- Quit/Close when a reliable target exists

A window-preview/selection popout for multi-window apps may be added using the same popout primitive.

## drag and reorder

Pinned items support direct drag reorder.

Requirements:

- reorder updates the canonical favourites order
- dragging a transient running item does not silently pin it
- explicit pinning inserts it into the pinned section
- drag feedback uses shared motion/material primitives
- releasing outside the dock cannot corrupt ordering

Drag-to-unpin may exist later but must not be the only unpin mechanism.

## dock visibility

Supported modes:

```text
persistent
auto-hide
smart
```

Default: `smart`.

`persistent` reserves a stable bottom exclusive zone.

`auto-hide` overlays windows and reveals through a narrow bottom hot zone.

`smart` overlays when the dock would obstruct relevant window content and may remain visible when unobstructed.

Fullscreen always hides the dock.

In `auto-hide` and `smart`, do not resize compositor work area on every reveal. Window reflow on pointer entry is unacceptable.

Reveal/hide should animate the material surface as a coherent sheet with opacity/position/shape continuity.

## multi-monitor

Top bar may run on every eligible monitor.

Dock monitor modes:

```text
primary
all
focused
```

Default: `primary`.

Pinned state is global.

Running state is derived from canonical app windows.

Focus emphasis follows the active toplevel.

Monitor hotplug must not require restarting Caelestia.

## popouts

Dock tooltips, window lists, status menus and context menus reuse Caelestia popout infrastructure but render with the shared Liquid Glass primitive.

Popouts must:

- clamp to monitor bounds
- stay above the owning surface
- visually connect to their source without fake arrows when unnecessary
- survive dock magnification without positional jumping
- close predictably
- not steal keyboard focus merely because the pointer entered the dock

Nested popouts should avoid stacking multiple heavy blur/refraction layers over each other. Prefer one clear hierarchy of materials.

## palette semantics

All semantic state colours come from the existing Caelestia/Vesper palette.

The material renderer needs roles equivalent to:

```text
material tint
on-material foreground
secondary foreground
focused accent
running neutral
urgent semantic colour
shadow luminance
specular luminance
```

Material tint is not the same thing as accent colour.

The active accent drives focused/selected emphasis. It should not flood the whole glass surface.

Light/dark wallpaper conditions may affect material tint and contour strength independently from semantic accent.

## reduced transparency and fallback

Liquid Glass must degrade safely.

Reduced-transparency mode:

- disables or greatly reduces refraction
- raises material opacity
- may reduce backdrop blur cost
- preserves semantic tint and contour contrast
- keeps foreground readability

If the graphics stack cannot provide the required backdrop sampling/effect reliably, fall back to a high-quality static translucent material rather than showing broken shaders or missing surfaces.

Fallback must still use the shared palette and geometry.

## performance

The shell is event-driven.

Use existing signals/models for:

- Hyprland toplevel creation/removal/focus
- desktop-entry changes
- launcher favourite changes
- adaptive-icon changes
- palette/theme changes
- monitor changes

Do not run fixed interval processes to rebuild the dock model.

Expensive icon generation belongs to the adaptive-icon service, not QML.

Material rendering requirements:

- avoid separate full-screen blur passes per dock item
- reuse one backdrop/material pass per coherent surface where practical
- stop unnecessary animations/effects while hidden
- cache static geometry where it does not break dynamic material behaviour
- preserve smooth pointer interaction while magnification is active

Do not sacrifice shell input latency for visually stronger refraction.

## declarative configuration target

The exact upstream schema may differ. Vesper should expose a model equivalent to:

```nix
programs.caelestia.settings = {
  topBar = {
    enabled = true;
    position = "top";
    persistent = true;
    monitors = "all";

    material = {
      type = "liquid-glass";
      variant = "regular";
    };
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

    material = {
      type = "liquid-glass";
      variant = "regular";
    };

    magnification = {
      enabled = true;
      maxScale = 1.18;
      neighbourScale = 1.08;
    };
  };
};
```

This describes Vesper's public target, not the current upstream Caelestia schema.

Do not expose raw shader/refraction parameters in normal settings.

## settings UI

Caelestia settings should expose only useful policy controls.

Dock controls:

- visibility: persistent / smart / auto-hide
- icon size
- magnification on/off and strength
- monitor mode
- show running unpinned applications
- reset pinned order to launcher favourites
- Liquid Glass variant: regular / clear when clear is supported safely

Top bar controls may include monitor policy and layout ordering.

Do not expose independent dock/bar colour pickers.

Accent and appearance remain owned by the existing Vesper theme controls.

## implementation shape

Prefer explicit separation of model, shell geometry and material rendering.

Conceptually:

```text
Caelestia shell
├── LiquidGlassSurface
│   ├── backdrop/blur
│   ├── refraction
│   ├── tint/contrast
│   ├── contour/specular
│   └── shadow
│
├── TopBar
│   ├── start section
│   ├── centre section
│   └── end section
│
├── VesperDock
│   ├── DockModel
│   ├── DockItem
│   ├── DockContextPopout
│   └── DockVisibilityController
│
└── shared services
    ├── Hyprland state
    ├── DesktopEntries/AppDB
    ├── canonical Vesper app identity
    ├── Vesper adaptive icons
    └── Colours/Tokens
```

If extending Caelestia's C++ config layer is necessary, isolate those changes by responsibility. Do not scatter `position`, dock or Liquid Glass constants through unrelated QML files.

## implementation order

Implement in this order:

1. horizontal Caelestia bar orientation and start/centre/end sections
2. top bar geometry using existing entries
3. shared canonical dock model using launcher favourites + Hyprland toplevels
4. bottom centred dock with correct focus/launch/running behaviour
5. canonical adaptive-icon consumption
6. shared Liquid Glass material primitive
7. apply Liquid Glass to top bar and dock
8. pointer-distance magnification
9. context popout and pin/reorder behaviour
10. smart hide and multi-monitor policy
11. reduced-motion/reduced-transparency fallbacks
12. settings UI

Functional application identity and shell behaviour must work before visual shader tuning is considered finished.

## acceptance criteria

The implementation is not complete until all of these hold:

- no left-side taskbar remains in the default Vesper layout
- top system bar is horizontal and usable
- dock is bottom-centred and content-sized
- top bar and dock are native Caelestia/Quickshell surfaces
- no Waybar/Plank/GTK/extra dock daemon is installed
- pinned and running apps deduplicate correctly
- focus-or-launch works for normal applications
- canonical identity is reused instead of duplicated
- adaptive icons appear consistently across launcher and dock
- dock reorder persists through the canonical favourites source
- fullscreen hides the dock
- smart/auto-hide does not cause window reflow on every reveal
- multi-monitor hotplug does not require shell restart
- palette/accent changes propagate without a second theme system
- the material visibly contains blur, restrained optical displacement/refraction, adaptive tint, contour/specular response and depth shadow
- the dock does not look like a plain translucent rectangle
- foreground icons/text are never included in the blur/refraction pass
- nested per-item glass cards are not used as the default dock design
- reduced-transparency has a readable safe fallback
- reduced-motion disables magnification/spring motion
- hidden surfaces do not continue expensive animation work
- configured Caelestia package builds successfully
- complete Home Manager/system evaluation still succeeds

## final design rule

If a visual choice conflicts with the Liquid Glass material model, the Liquid Glass model wins.

If a visual choice conflicts with application identity, accessibility, input latency or shell reliability, correctness wins and the material effect degrades gracefully.

The result should look like Apple Liquid Glass adapted to a native Linux/Hyprland shell, not like a Linux dock with blur turned on.