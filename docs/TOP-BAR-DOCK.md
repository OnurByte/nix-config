# Apple-aligned top bar + Liquid Glass dock

Status: **plan**

Do not implement this document unless the task explicitly activates the plan.

This document is the visual authority for the planned Vesper top-bar and dock redesign.
It corrects older generic glass assumptions but does not override current code merely by existing.

## target

```text
transparent macOS-style top bar
+
Apple-aligned Liquid Glass application dock
+
Liquid Glass only on appropriate transient/navigation surfaces
```

The target is not generic glassmorphism, not a visionOS imitation and not one frosted-panel recipe applied everywhere.

The implementation remains Caelestia/Quickshell native.
Do not add another shell, Waybar, Plank, a GTK dock or a second application identity system.

## authority

Use this order for visual decisions in this plan:

1. current Apple HIG / public Liquid Glass guidance
2. current Apple macOS examples and official design sessions
3. Vesper-specific adaptation for Quickshell/Qt
4. current Vesper/Caelestia architecture
5. third-party implementations only as engineering references

Do not invent private Apple shader internals.
Match observable hierarchy and behavior rather than claiming a fake reverse-engineered rendering pipeline.

## shell hierarchy

### persistent top bar

The top bar is transparent.
It is not a persistent glass slab.

Suggested structure:

```text
start                                centre                               end
logo  workspaces  active window      clock       privacy  AI  tray  status  power
```

Foreground content must remain legible against arbitrary wallpaper/window content through adaptive foreground treatment, restrained local shadows/contrast aids and accessibility modes.

Do not solve difficult backgrounds by turning the whole bar into opaque acrylic.

### persistent dock

The dock is the main persistent Liquid Glass surface.

It contains applications only.
Pinned applications preserve configured order.
Running unpinned applications appear after pinned applications.
Pinned + running resolves to one item.

Use `launcher.favouriteApps` as the initial pin source instead of creating a second favourites database.

Default material: `Regular` Liquid Glass.

Do not use permanent child glass tiles behind each icon.
The dock should read as one coherent material surface.

### transient surfaces

Menus, context menus, status popouts, control-center panels, overflow surfaces and similar transient navigation/control presentations may use Liquid Glass.

They should appear spatially connected to the control that invoked them.
Avoid glass-on-glass nesting inside the popout.

## material rules

Liquid Glass is not defined as `blur + opacity + border`.
The Vesper approximation should support observable behavior such as:

- lensing / optical deformation
- background-aware adaptation
- geometry-aware highlight and shadow
- size/role-dependent material response
- interaction-driven emphasis
- accessibility-driven changes
- spatial morphing where practical

Do not use a permanently bright luminous outline as the definition of glass.
Highlights must be restrained and context-aware.

### Regular vs Clear

Liquid Glass material variants are:

```text
Regular
Clear
```

`Regular` is the default.
Use `Clear` only when foreground legibility and the content beneath the material make it appropriate.

Do not continuously blend `Clear` into `Regular` as an automatic contrast rescue.
Choose the semantic variant deliberately.

## icon appearance is separate

Adaptive icon appearance is governed only by `ADAPTIVE-ICONS.md` and current Vesper theme state.

Current appearance names are:

```text
Automatic
Default
Dark
Tinted
Clear
```

Liquid Glass material variants are a different namespace:

```text
Regular
Clear
```

Never automatically map:

```text
Clear icon appearance -> Clear dock material
Tinted icon appearance -> tint the entire dock
```

Icon material/rendering controls remain separate from shell Liquid Glass material selection.

## tint and accent

Keep normal shell glass neutral/contextual.
Use accent tint selectively for meaningful focus, selection or primary emphasis.

Do not tint the entire dock simply because an accent color exists.
Do not use neon source colors or multi-color glowing borders.

## geometry

Use concentric geometry.

- capsules where the component naturally fits a capsule
- rounded rectangles for larger/denser surfaces
- inner radii derived from outer radius and padding
- no pinched or independently chosen nested corners

The dock is bottom anchored, horizontally centered and content sized.

## motion

Prefer spatial continuity and shape/depth response over fade-only transitions.

Useful behavior may include:

- restrained dock magnification
- geometry interpolation
- source-to-popout spatial continuity
- subtle depth/lensing response

Opacity fades may support a transition but should not be the entire interaction language.

## accessibility

### reduced transparency

Increase separation/obscuration and foreground legibility.
Do not simply remove all material behavior.

### increased contrast

Use stronger black/white foreground/material treatment and contrasting edges when required.
Do not rely on subtle wallpaper-derived contrast.

### reduced motion

Disable or heavily reduce:

- elastic deformation
- dock magnification
- bounce/spring effects
- energetic morphing

Keep state changes clear with short/simple transitions.

## implementation boundary

When this plan is activated:

1. inspect the current Caelestia bar/dock architecture first
2. preserve one shell and one application identity model
3. introduce shared semantic material tokens rather than copied constants
4. keep top bar and dock behavior component-specific
5. avoid glass-on-glass nesting
6. preserve adaptive icon ownership in `ADAPTIVE-ICONS.md`
7. validate against bright, dark, high-frequency and mixed-content backgrounds
8. test reduced transparency, increased contrast and reduced motion states

Do not implement this plan piecemeal from isolated screenshots or old visual assumptions.
