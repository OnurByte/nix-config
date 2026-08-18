# Apple-aligned top bar + Liquid Glass dock

Status: **design and implementation plan only — do not implement from this document yet**

This document replaces the earlier generic “Liquid Glass everywhere” interpretation with a plan grounded in Apple’s public Liquid Glass guidance and Apple’s own macOS Tahoe 26 examples.

The target is not “glassmorphism”, not a visionOS approximation, and not a custom frosted-panel theme with Apple terminology added on top.

The target is:

```text
transparent macOS-style top system bar
+
Apple-aligned Liquid Glass application dock
+
Liquid Glass only for the transient/navigation surfaces where it is appropriate
```

The implementation remains native to the existing Caelestia/Quickshell shell. This document plans the visual and interaction behavior only. It does not authorize adding another shell, GTK dock, Waybar, Plank, a new daemon, or a second application/icon identity system.

---

## 1. source of truth

When this document and an older Vesper visual assumption disagree, use this order:

1. Apple Human Interface Guidelines and current Apple Developer Liquid Glass documentation.
2. Apple WWDC25 design sessions and Apple’s official sample code.
3. Apple’s macOS Tahoe 26 system examples.
4. Vesper-specific adaptation needed because the implementation is Quickshell/Qt rather than SwiftUI/AppKit.
5. Prior third-party Quickshell/Caelestia implementations only for architecture, never as the visual authority.

Primary Apple references reviewed for this revision:

- Apple HIG — **Materials**
  - https://developer.apple.com/design/human-interface-guidelines/materials
- Apple Developer — **Liquid Glass**
  - https://developer.apple.com/documentation/technologyoverviews/liquid-glass
- Apple Developer — **Adopting Liquid Glass**
  - https://developer.apple.com/documentation/technologyoverviews/adopting-liquid-glass
- Apple Developer — **Applying Liquid Glass to custom views**
  - https://developer.apple.com/documentation/swiftui/applying-liquid-glass-to-custom-views
- Apple Developer — **Landmarks: Building an app with Liquid Glass**
  - https://developer.apple.com/documentation/swiftui/landmarks-building-an-app-with-liquid-glass
- WWDC25 — **Meet Liquid Glass**
  - https://developer.apple.com/videos/play/wwdc2025/219/
- WWDC25 — **Get to know the new design system**
  - https://developer.apple.com/videos/play/wwdc2025/356/
- WWDC25 — **Build a SwiftUI app with the new design**
  - https://developer.apple.com/videos/play/wwdc2025/323/
- Apple — **macOS Tahoe 26 makes the Mac more capable, productive, and intelligent than ever**
  - https://www.apple.com/newsroom/2025/06/macos-tahoe-26-makes-the-mac-more-capable-productive-and-intelligent-than-ever/
- Apple — **Apple introduces a delightful and elegant new software design**
  - https://www.apple.com/newsroom/2025/06/apple-introduces-a-delightful-and-elegant-new-software-design/

The Apple APIs named in those documents are references for behavior and hierarchy. Vesper is **not** expected to recreate SwiftUI APIs literally.

---

## 2. the biggest correction: the top bar is not a Liquid Glass strip

The previous version treated both the top bar and dock as two similar glass surfaces.

That conflicts with Apple’s macOS Tahoe design.

Apple explicitly presents the macOS menu bar as completely transparent while the Dock, sidebars, toolbars, controls, menus and other floating functional elements receive the new Liquid Glass treatment.

Therefore the Vesper target becomes:

```text
┌──────────────────────────────────────────────────────────────────┐
│ transparent top bar                                              │
│ logo/workspaces/window       clock       privacy/status/power   │
└──────────────────────────────────────────────────────────────────┘


                         application space


                  ╭──────────────────────────╮
                  │   Liquid Glass app dock  │
                  ╰──────────────────────────╯
```

The top bar must not look like a floating frosted capsule, a translucent Waybar block, or a second copy of the dock.

The top bar is a transparent functional plane with foreground content and adaptive legibility treatment.

Liquid Glass is used when the top bar opens a menu, popover, Control-Center-style panel, overflow surface, contextual control or another transient navigation/control surface.

### Vesper decision

The Vesper shell keeps the user’s desired top-bar + bottom-dock layout, but follows the current macOS distinction:

- top bar: transparent
- dock: Liquid Glass
- transient menus/popouts: Liquid Glass
- normal app/window content: not Liquid Glass merely for decoration

---

## 3. what Apple actually means by Liquid Glass

Liquid Glass is not defined by one blur radius, opacity value or shader recipe.

Apple describes it as a dynamic digital material whose visual and motion behavior work together.

The public behavior that matters to Vesper is:

```text
lensing / light bending
reflection and refraction
background-aware appearance
tint and dynamic-range adaptation
context-aware shadow/depth
geometry-aware highlights
light/dark legibility adaptation where appropriate
interaction-driven illumination/flex
fluid morphing between related controls and presentations
size-dependent material character
accessibility-driven material changes
```

Apple describes lensing as the primary visual way the material defines itself. Previous materials primarily scattered light; Liquid Glass also bends, shapes and concentrates it.

Do not reduce the implementation plan to:

```text
blur + opacity + 1px border
```

But also do not pretend Apple has published an exact internal shader pipeline. It has not.

### corrected Vesper rule

Plan around **observable behavior**, not a fake reverse-engineered Apple rendering pipeline.

It is acceptable for Vesper to approximate the effect using Qt/Quickshell primitives and a small shader/effect layer, but the spec must not claim that a particular ordered list of blur/refraction/specular stages is Apple’s implementation.

---

## 4. corrections to the previous spec

### 4.1 top bar

Old assumption:

```text
Liquid Glass top strip
```

Corrected plan:

```text
transparent top system bar
```

Do not put one persistent glass slab behind the entire top bar.

### 4.2 identical material recipe everywhere

Old assumption:

```text
one identical glass recipe for dock, bar, menus and popouts
```

Corrected plan:

Use one shared **Liquid Glass material model and token system**, but allow its behavior to vary by component size, role, focus and background.

Apple explicitly changes the perceived thickness and optical behavior of larger glass surfaces. A menu expanded from a small control can use deeper shadow, stronger lensing/refraction and softer scattering than the smaller source control.

Therefore shared code must not mean identical constants.

### 4.3 Regular and Clear

Old assumption:

```text
Clear can automatically drift/fallback toward Regular as contrast changes
```

Corrected plan:

Do not continuously blend or mix the two semantic variants.

Apple defines:

- `regular`: the normal, adaptive, broadly usable variant
- `clear`: permanently more transparent, without the same adaptive behavior

Apple explicitly says the variants should not be mixed.

`regular` is the Vesper default.

Use `clear` only when all of the following are true:

1. the glass is over visually/media-rich content;
2. adding the required dimming treatment does not harm the content layer;
3. the foreground content above the clear glass is bold and bright enough to remain legible.

If those requirements are not guaranteed, choose `regular` before rendering. Do not animate a live mixture between Clear and Regular to rescue contrast.

### 4.4 accent and tint

Old assumption:

```text
current accent colour lightly tints the complete dock/bar material
```

Corrected plan:

Apple recommends tinting selectively for prominence, not washing the entire navigation layer in the accent colour.

Therefore:

- normal dock glass remains neutral/contextual;
- accent tint is reserved for primary actions or meaningful selected/focused emphasis;
- do not tint every dock item;
- do not tint the complete dock background merely because the user selected an accent colour;
- normal running state should stay neutral;
- urgent state uses semantic urgency colour;
- focused/selected state can use accent deliberately.

### 4.5 icon tint is not material tint

Vesper has adaptive icon appearances such as Light, Dark, Tinted and Clear.

These are **not the same concept** as the `regular` and `clear` Liquid Glass material variants.

Keep the namespaces separate:

```text
icon appearance
    Original / Light / Dark / Tinted / Clear

Liquid Glass variant
    Regular / Clear
```

Never automatically map:

```text
Clear icon appearance -> Clear dock material
Tinted icon appearance -> tint entire dock material
```

The adaptive-icon system remains governed by `docs/ADAPTIVE-ICONS.md`.

### 4.6 static optical recipe

Old assumption:

```text
fixed blur
fixed contour highlight
fixed shadow
fixed refraction amount
```

Corrected plan:

Material response is context dependent.

The implementation plan must allow appearance to adapt based on:

- what lies behind the glass;
- whether the material is small or large;
- active/inactive focus state;
- interaction state;
- accessibility settings;
- display/performance conditions.

### 4.7 fade-only transitions

Old assumption:

```text
show/hide primarily through opacity fades
```

Corrected plan:

Apple describes Liquid Glass as materializing through changing lensing/light behavior and morphing spatially between related surfaces.

Vesper should therefore prefer:

- shape/geometry interpolation;
- depth/lensing response changes;
- spatial continuity from source control to presentation;
- restrained opacity only as a supporting effect.

A pure cross-fade is a fallback, not the target interaction language.

### 4.8 nested glass

Old assumption:

```text
shared dock glass + local glass lens behind each hovered icon
```

This risks glass-on-glass.

Apple explicitly says to avoid stacking Liquid Glass on Liquid Glass.

Corrected plan:

- the dock normally has one coherent Liquid Glass surface;
- icon foregrounds do not each get their own permanent glass tile;
- hover/press feedback should modulate the shared material or use foreground fill/vibrancy/highlight;
- menus/popovers become a separate presentation when opened, with a clear spatial relationship to their source;
- elements placed on glass should generally use fills, transparency and vibrancy instead of another full glass material.

### 4.9 border-as-glass

Old assumption:

```text
strong luminous border is a core defining feature
```

Corrected plan:

Highlights are part of the material response, but Apple’s design is not a static glowing outline.

Use geometry- and environment-aware highlights and shadows. Avoid a permanently bright white ring or decorative neon edge.

### 4.10 reduced transparency

Old assumption:

```text
reduced transparency = disable blur and flatten material
```

Corrected plan:

Apple’s Reduced Transparency treatment makes Liquid Glass frostier and obscures more of what is behind it.

Vesper’s accessibility mode should therefore increase separation/obscuration and legibility instead of merely removing all material behavior.

### 4.11 increased contrast

Plan for an increased-contrast state where foreground/material treatment becomes much more explicitly black/white and receives a contrasting edge where necessary.

Do not rely on subtle tint, lensing or wallpaper-derived contrast when increased contrast is requested.

### 4.12 Reduced Motion

Apple reduces the intensity of effects and disables elastic properties.

Vesper should disable or heavily reduce:

- elastic material deformation;
- dock magnification;
- bounce/spring effects;
- energetic morphing.

Preserve state changes and spatial clarity with short/simple transitions.

### 4.13 arbitrary squircle geometry

Old assumption:

```text
custom squircle-derived shapes everywhere
```

Corrected plan:

Apple’s current system language is strongly based on concentric geometry.

Use:

- capsule shapes where a capsule makes sense;
- rounded rectangles for denser/larger desktop surfaces where a capsule would be visually wrong;
- inner radii derived concentrically from parent radius and padding;
- no pinched or flared nested corners.

On macOS, dense small/medium controls can remain rounded rectangles; capsules are better reserved for larger or standout controls.

---

## 5. target shell hierarchy

The shell should have three visual levels.

### level A — content

```text
wallpaper
application windows
normal desktop content
```

Do not place Liquid Glass here merely for decoration.

### level B — persistent functional shell

```text
transparent top bar
Liquid Glass dock
```

The dock forms a distinct floating functional layer above content.

The top bar remains visually lighter and more open, matching the current macOS menu-bar direction.

### level C — transient functional presentations

```text
menus
context menus
control-center panels
status popouts
overflow menus
window chooser popouts
power/session popouts
```

These may use Liquid Glass when they are the active navigation/control presentation.

They should appear spatially connected to the control that invoked them.

---

## 6. transparent top bar plan

The existing left-side Caelestia bar becomes a horizontal transparent top bar.

Default structure:

```text
start                                centre                               end
logo  workspaces  active window      clock       privacy  AI  tray  status  power
```

The top bar itself has no persistent frosted background.

### legibility

Foreground labels and symbols need enough contrast against arbitrary wallpaper/window content.

Plan for adaptive foreground treatment rather than a large permanent background slab.

Possible Vesper techniques, to validate visually rather than assume:

- adaptive light/dark foreground role;
- restrained text/icon shadow only when needed;
- localized selection/hover treatment;
- accessibility contrast mode;
- optional very subtle edge/legibility treatment behind a control only when the control becomes interactive.

Do not solve a difficult wallpaper by turning the full top bar into opaque acrylic.

### grouping

Apple’s current toolbar guidance emphasizes hierarchy through layout and grouping instead of decorative backgrounds.

Apply the same principle:

- related system controls are spatially grouped;
- unrelated controls receive visible spacing;
- secondary telemetry goes into overflow before the bar becomes crowded;
- important privacy/network/battery/power information has higher persistence priority;
- text and symbols should not be packed so closely that they appear to be a single action.

### popouts

A click on network, Bluetooth, power, privacy, AI or another top-bar control may open a Liquid Glass popout.

The popout should:

- originate visually from the invoking control;
- use a larger-material treatment than a tiny button;
- avoid glass-on-glass inside itself;
- use ordinary fills/vibrancy for rows, selections and secondary elements;
- remain legible over arbitrary desktop content.

---

## 7. Liquid Glass dock plan

The bottom dock is the main persistent Liquid Glass surface.

It contains applications only.

```text
launcher favourites
        +
running applications
        ↓
canonical Vesper app identity
        ↓
one item per application
```

Pinned applications preserve configured order.

Running unpinned applications appear after pinned applications.

Pinned + running resolves to one dock item.

Use `launcher.favouriteApps` as the initial pin source instead of creating a second favourites database.

### default material

Default dock material: **Regular Liquid Glass**.

Do not use Clear by default simply because it looks more transparent in a screenshot.

The default dock must remain usable against:

- bright wallpaper;
- dark wallpaper;
- high-frequency wallpaper;
- windows behind the dock;
- mixed light/dark content.

### dock shape

The dock is bottom anchored, horizontally centred and content sized.

The outer surface should read as one coherent rounded glass body.

Use a capsule-like geometry when the dock is short enough for that geometry to remain natural.

For larger/overflowing dock configurations, preserve concentric rounded geometry rather than forcing an over-inflated capsule.

Inner spacing/radii must be derived from outer geometry rather than chosen independently.

### no permanent child glass tiles

Normal dock item:

```text
app artwork
running/focus state
hit target
```

It does **not** need its own glass capsule.

This keeps the hierarchy clear and avoids stacking glass within glass.

---

## 8. dock material behavior

### lensing

Lensing is a defining behavior, but Vesper should reproduce it conservatively.

Plan for an optical deformation that communicates material presence while preserving wallpaper recognition and pointer precision.

Do not specify an invented Apple formula such as “refraction is always strongest exactly N pixels from the contour”.

Acceptance is perceptual:

- the glass reads as optically present, not merely blurred;
- the effect does not become a water/ripple filter;
- foreground app icons remain undistorted;
- static pointer state does not produce wobbling noise.

### background-aware adaptation

Regular glass should adapt to the content beneath it.

Plan for a lightweight backdrop analysis sufficient to influence:

- foreground light/dark role;
- shadow separation;
- tint/dynamic-range compensation;
- highlight strength;
- material opacity/scattering within a bounded design range.

Do not expose raw versions of these values as normal user settings.

### shadows

Apple demonstrates shadow strength changing based on the content beneath a glass surface.

Vesper should plan for content-aware separation rather than one fixed heavy shadow.

Examples:

- over text/high-frequency background: stronger separation may be needed;
- over a simple light field: less shadow may be needed;
- larger popout: deeper/richer shadow than the dock’s small steady-state treatment.

### highlights

Highlights follow geometry and help define the silhouette.

They should not become a static 1 px white stroke.

Plan for:

- shape-aware highlight response;
- restrained environmental light relationship;
- interaction-driven illumination;
- no neon RGB/chromatic border gimmick.

### ambient colour

Apple shows larger glass surfaces picking up subtle influence from nearby colourful content.

Vesper may approximate this with restrained backdrop chroma sampling.

This is different from applying the user accent colour to the whole surface.

---

## 9. pointer interaction and material response

Apple’s custom Liquid Glass supports interactive behavior for touch and pointer input.

On interaction, the material can flex and energize with light.

For Vesper dock hover:

```text
pointer approaches icon
        ↓
icon magnification begins
        ↓
shared dock material receives restrained local illumination/deformation
        ↓
foreground state remains crisp
```

Do not spawn another independent Liquid Glass tile underneath the hovered icon.

### press

Press feedback can combine:

- slight foreground scale compression;
- localized material illumination;
- small shape/depth response;
- short transition into the resulting action/presentation.

Avoid generic button-opacity blinking if a spatial/material response is available.

---

## 10. dock magnification

Dock magnification is a macOS interaction reference, not part of the definition of Liquid Glass itself.

Keep it as an independent interaction feature.

Planning baseline:

```text
hovered app              ~1.18 max
nearest neighbour        ~1.08
far apps                 1.00
```

These are Vesper tuning targets, not Apple-published metrics.

Requirements:

- continuous pointer-distance curve;
- predictable hit targets;
- no icon collisions;
- coherent container resizing where needed;
- no compositor exclusive-zone changes on every pointer movement;
- Reduced Motion disables magnification.

Do not confuse “fluid” with exaggerated bouncing.

---

## 11. app states without glass-on-glass

A dock item may represent:

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

State treatment belongs primarily to foreground content and lightweight overlays.

### running

Use a small neutral indicator.

Do not make the running indicator another Liquid Glass object.

### focused

Use the semantic accent deliberately.

Possible treatment:

- brighter/wider running indicator;
- subtle accent emphasis;
- foreground symbol treatment.

Do not tint the entire dock merely because one app is focused.

### urgent

Use semantic urgency colour in a bounded way.

Urgency must not erase focused/running information.

### launching

Use a small local progress/spinner treatment, then clear when a matching toplevel appears or launch fails/times out.

---

## 12. menus, context menus and morphing

One of the strongest Apple examples is a control that fluidly expands/morphs into the menu or popover it presents.

Vesper should plan the same spatial relationship for:

- dock right-click menu;
- multi-window chooser;
- top-bar status popouts;
- power/session menu;
- Control-Center-style surfaces.

The user should be able to visually understand:

```text
this control
    ↓
opened this surface
```

rather than seeing an unrelated floating rectangle appear elsewhere.

### larger material behavior

When a small source expands into a larger presentation, the larger surface should read as more substantial.

Plan for stronger depth/separation on large presentations than on the small source.

Do not simply scale a tiny dock-button shader texture to menu size.

---

## 13. Apple sample patterns to copy conceptually

These are behavior references, not SwiftUI implementation instructions.

### 13.1 `glassEffect()` default behavior

Apple’s custom-view API uses Regular glass by default and a capsule as the default effect shape.

Vesper lesson:

- Regular is the safe default;
- start from simple system-like geometry;
- introduce a rounded rectangle for larger components rather than forcing every surface into a capsule.

### 13.2 `interactive()`

Apple can make custom glass respond to touch and pointer input.

Vesper lesson:

- pointer response is part of the material behavior;
- hover/press should not be represented only by changing the icon’s opacity.

### 13.3 `GlassEffectContainer`

Apple groups related glass effects so they can combine and morph coherently.

Vesper lesson:

- related transient glass shapes need one coordination context;
- do not instantiate many unrelated heavy material containers;
- maintain spatial continuity between source and destination surfaces.

This does **not** override Apple’s rule against visually stacking independent sheets of glass on top of each other.

### 13.4 Landmarks badges

Apple’s Landmarks sample uses coordinated IDs and a glass container so custom badge elements can morph smoothly.

Vesper lesson:

- stable identity is useful for morphing;
- dock item -> context menu/window chooser transitions can share a source identity;
- state transitions should preserve object continuity.

### 13.5 toolbar grouping

Apple’s new toolbar design groups related actions into shared visual groups and separates primary actions.

Vesper lesson for the top bar:

- group by function and frequency;
- remove low-value clutter before adding more decoration;
- keep primary/high-priority controls clear;
- do not wrap every icon in its own permanent background.

### 13.6 macOS menu bar

Apple’s macOS Tahoe example makes the menu bar completely transparent.

Vesper lesson:

- top system bar background should disappear;
- visual weight belongs to the dock and transient controls, not a second persistent glass slab.

### 13.7 macOS Dock

Apple describes the Dock as part of the new Liquid Glass system experience while preserving its familiar role.

Vesper lesson:

- keep the dock recognizably a dock;
- do not turn it into a telemetry/dashboard panel;
- app launching/switching stays the only persistent role.

---

## 14. Clear variant plan

Clear is an optional experiment, not the default visual target.

Do not add a generic `glassOpacity = 0.1` setting and call it Clear.

A future Clear dock mode can be considered only after Regular is correct.

Validation conditions:

```text
media-rich/wallpaper-rich background     yes
localized/global dimming is acceptable   yes
foreground icons/state are bold/bright   yes
```

If any answer is no, Clear should not be offered for that component/state.

Do not mix Regular and Clear within one dock material group.

---

## 15. color and Vesper palette plan

The existing Vesper palette remains the semantic color source for shell state.

But Liquid Glass is not a big palette-colored pane.

Separate four concepts:

```text
1. environment/backdrop influence
2. semantic UI state color
3. Liquid Glass tint for prominence
4. adaptive app-icon appearance
```

They must not collapse into one `accentColor` multiplier.

### normal dock

- neutral/context-derived material
- readable foreground
- no accent wash

### focused app

- semantic accent can emphasize focus

### primary action in a popout

- may use Liquid Glass tint deliberately

### icon Tinted appearance

- handled by adaptive icon pipeline
- can follow selected user palette/accent according to `ADAPTIVE-ICONS.md`
- does not force the dock material to use the same tint

---

## 16. content hierarchy and “use glass sparingly”

Apple repeatedly warns against overusing Liquid Glass.

Vesper should therefore avoid applying the material to:

- normal settings cards;
- every telemetry card;
- app content backgrounds;
- every dock item;
- every top-bar icon;
- nested controls inside an already-glass panel.

Good Vesper candidates:

- dock outer surface;
- context menu;
- window chooser;
- Control-Center-style panel;
- transient status popout;
- selected custom control only when its interaction actually benefits from the material.

The underlying desktop/app content should remain the visual focus.

---

## 17. accessibility plan

The custom Quickshell implementation must have explicit equivalents of Apple’s accessibility adaptation.

### Reduced Transparency

Target:

- frostier/more obscuring material;
- higher content separation;
- less dependence on live wallpaper detail;
- maintain recognizable component geometry.

### Increased Contrast

Target:

- stronger black/white foreground choice;
- more explicit contrast boundary where needed;
- less dependence on subtle tint/highlight behavior.

### Reduced Motion

Target:

- dock magnification disabled;
- elastic/flex motion disabled or heavily reduced;
- morphing simplified;
- no bounce-heavy launch animation;
- state transitions remain obvious.

These modes are part of acceptance criteria, not optional polish.

---

## 18. performance plan

Apple warns that too many custom Liquid Glass effects/containers can degrade performance.

Vesper should plan around a small number of expensive material regions.

Steady desktop target:

```text
transparent top bar             cheap
one Liquid Glass dock           primary persistent material cost
transient popout                only while open
```

Do not render one custom refraction shader per dock icon.

Do not create dozens of hidden live glass surfaces.

### quality tiers

Plan a graceful quality ladder instead of changing the visual design entirely:

```text
full
    adaptive lensing/refraction + highlights + shadows

reduced
    simplified optical deformation + adaptive material + shadow/highlight

safe
    standard blur/material approximation + strong legibility

accessibility
    frosted/high-contrast variant according to user settings
```

The hierarchy, shapes and interactions must survive every tier.

---

## 19. app identity and icons remain unchanged architecturally

Liquid Glass work must not create a parallel dock identity system.

The dock still consumes the canonical application identity from `docs/ADAPTIVE-ICONS.md`.

Identity can reconcile:

```text
desktop id
StartupWMClass
Wayland app_id
X11 WM_CLASS
Flatpak id
executable
Electron app id
Steam app id
Wine/Proton identity
browser PWA identity
explicit compatibility aliases
```

Invariant:

```text
launcher icon == dock icon == running-state icon == app-grid icon
```

The dock uses the active Vesper adaptive icon result and falls back to the real desktop-entry icon if necessary.

Do not duplicate icon-generation rules in this document.

---

## 20. dock application behavior plan

Primary click:

- not running -> launch
- one matching window -> focus
- multiple matching windows -> focus most recently active matching window

Clicking the already-focused app does not implicitly minimize it.

Middle click may request a new instance when supported.

Right click opens a context presentation spatially connected to the dock item.

Possible actions:

- New Window
- running windows
- Pin / Unpin
- desktop-entry actions
- Quit/Close when a reliable target exists

Pinned items support drag reorder.

A running unpinned app must not silently become pinned merely because it was dragged through the transient running section.

---

## 21. visibility plan

Dock modes remain:

```text
persistent
auto-hide
smart
```

Default target: `smart`.

### persistent

Stable bottom exclusive zone.

### auto-hide

No permanent exclusive zone; narrow reveal area.

### smart

Visible when it does not obstruct relevant content, hidden/overlaid when it would.

Fullscreen hides the dock.

Auto-hide/smart must not resize the compositor work area on every pointer reveal.

### reveal motion

Target behavior:

- spatial/material continuity;
- restrained position and lensing/depth change;
- no fade-only appearance as the primary effect;
- no excessive bounce.

---

## 22. multi-monitor plan

Top bar may exist on every eligible monitor.

Dock modes:

```text
primary
all
focused
```

Default: `primary` until focused-monitor behavior is proven visually stable.

Pinned order is global.

Running/focus state comes from the canonical app/window model.

Monitor hotplug must not require shell restart.

Liquid Glass material sampling must remain local to the monitor containing the surface.

---

## 23. implementation architecture to plan toward

No code is requested by this document, but the eventual architecture should preserve these boundaries:

```text
Caelestia / Quickshell
├── TransparentTopBar
│   ├── start group
│   ├── centre group
│   ├── end group
│   └── transient Liquid Glass popouts
│
├── VesperDock
│   ├── DockModel
│   ├── DockItem
│   ├── DockVisibilityController
│   ├── DockPresentationCoordinator
│   └── Liquid Glass outer surface
│
├── LiquidGlassMaterialModel
│   ├── regular behavior
│   ├── optional clear behavior
│   ├── backdrop/context adaptation
│   ├── interaction response
│   ├── size/depth adaptation
│   └── accessibility/performance tiers
│
└── shared services
    ├── Hyprland window state
    ├── DesktopEntries/AppDB
    ├── canonical Vesper app identity
    ├── adaptive icon resolver
    ├── theme/palette semantics
    └── monitor/accessibility state
```

Important correction: `LiquidGlassMaterialModel` is a shared behavior model, **not a promise that every component uses the same exact visual constants**.

---

## 24. planning phases

### phase 0 — Apple reference lock

Before implementation, capture a small visual reference set from Apple’s official examples:

- macOS Tahoe transparent menu bar;
- macOS Tahoe Dock;
- a Regular Liquid Glass control over mixed content;
- a large menu/sidebar showing stronger depth;
- an interactive custom glass control;
- a control morphing into a menu/popover;
- Reduced Transparency / Increased Contrast behavior where available.

Create implementation acceptance screenshots from these references conceptually; do not pixel-copy assets.

### phase 1 — shell hierarchy cleanup

Plan changes required to:

- move current vertical bar to horizontal top layout;
- remove persistent bar background;
- separate dock from status/system controls;
- establish start/centre/end top-bar grouping;
- preserve existing Caelestia ownership and event sources.

No material work should compensate for a bad hierarchy.

### phase 2 — Regular Liquid Glass prototype

Prototype only the dock outer material.

Validate:

- lensing is perceptible but restrained;
- material remains readable over hostile wallpapers;
- highlights and shadows feel environmental rather than decorative;
- icons stay crisp;
- quality is stable during resize;
- no nested child glass.

Do not begin Clear until Regular passes.

### phase 3 — interaction response

Add pointer interaction to the existing coherent material:

- local illumination/flex;
- dock magnification;
- press response;
- state indicators.

Validate that interaction feels connected to the same surface rather than creating temporary floating cards.

### phase 4 — transient presentations

Plan source-connected transitions for:

- dock context menu;
- window chooser;
- top-bar status popouts;
- power/session surfaces.

Use larger-material behavior where appropriate.

### phase 5 — adaptive context and focus

Add/validate:

- background-aware foreground role;
- adaptive shadow/depth;
- active/inactive focus recession where useful;
- contextual highlight response.

Do not make a large popout rapidly flip its entire light/dark appearance if that would be distracting; Apple treats larger surfaces differently from smaller controls.

### phase 6 — accessibility

Validate Reduced Transparency, Increased Contrast and Reduced Motion before considering the design complete.

### phase 7 — performance

Measure material cost with:

- dock idle;
- dock magnification;
- dock + popout;
- multiple monitors;
- animated wallpaper/window content behind dock;
- reduced-quality fallback.

Limit persistent expensive surfaces.

### phase 8 — optional Clear experiment

Only after Regular is correct.

Clear must satisfy Apple’s usage conditions and remain a separate semantic variant rather than an opacity slider.

---

## 25. acceptance criteria

The plan is ready for implementation only when all of the following are agreed.

### hierarchy

- [ ] top bar is transparent rather than a permanent glass strip
- [ ] dock is the primary persistent Liquid Glass shell surface
- [ ] Liquid Glass is not applied throughout normal content
- [ ] transient controls clearly belong to the navigation/control layer

### material

- [ ] Regular is the default
- [ ] Clear is not mixed with Regular
- [ ] Clear is not treated as “Regular with lower opacity”
- [ ] lensing/refraction is present but does not distort foreground artwork
- [ ] material adapts to background/context
- [ ] shadow/highlight behavior is not a fixed decorative border recipe
- [ ] larger surfaces can look materially more substantial than smaller ones

### color

- [ ] dock glass is not globally accent tinted by default
- [ ] tint is reserved for meaningful prominence
- [ ] icon Tinted/Clear modes remain separate from Liquid Glass tint/variant
- [ ] semantic focus/running/urgent roles remain distinguishable

### structure

- [ ] no glass-on-glass dock item stack
- [ ] related controls use grouping/layout instead of unnecessary backgrounds
- [ ] geometry follows concentric/capsule/rounded-rectangle logic
- [ ] popouts remain spatially related to their source

### motion

- [ ] show/hide is not only a cross-fade
- [ ] menus/popovers can morph or spatially grow from their source where feasible
- [ ] pointer interaction can energize/flex the material without gimmicky wobble
- [ ] Reduced Motion removes elastic behavior and magnification

### accessibility

- [ ] Reduced Transparency produces a frostier/more obscuring treatment
- [ ] Increased Contrast does not depend on subtle wallpaper sampling
- [ ] Reduced Motion preserves clarity without spring effects

### performance

- [ ] one persistent dock material does not become N shaders for N app icons
- [ ] hidden/transient surfaces stop expensive work when inactive
- [ ] multi-monitor behavior remains bounded
- [ ] fallback quality tiers preserve hierarchy and legibility

### Vesper architecture

- [ ] Caelestia remains the only shell
- [ ] no GTK/Plank/Waybar/Latte/nwg-dock dependency
- [ ] app identity is shared with launcher/Apps/adaptive-icons
- [ ] icon rendering stays owned by the adaptive-icon system
- [ ] palette remains one semantic system
- [ ] no new polling loops where event-driven state exists

---

## 26. non-goals

This plan does not require:

- pixel-perfect reverse engineering of Apple private shaders;
- SwiftUI/AppKit dependency;
- Apple private APIs;
- copying Apple assets;
- a Liquid Glass effect on every shell component;
- a glass background behind the top bar;
- a second icon engine;
- a second app launcher database;
- a second shell process;
- a decorative neon-glass aesthetic.

The goal is to reproduce Apple’s **public design behavior and hierarchy** within Vesper’s native Linux/Quickshell architecture, while keeping the shell recognizably macOS-like in layout and recognizably Vesper in functionality.

---

## 27. final target in one sentence

**Transparent macOS-style top bar, one coherent Regular Liquid Glass application dock, source-connected Liquid Glass menus/popouts, adaptive optical behavior instead of generic blur, selective tint instead of accent wash, no glass-on-glass, and strict separation between Liquid Glass material variants and Vesper adaptive icon appearances.**
