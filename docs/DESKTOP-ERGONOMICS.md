# Desktop Ergonomics

Status: **implementation plan / spec**

This document is the canonical implementation plan for Vesper's high-frequency desktop ergonomics on top of the existing Caelestia + Hyprland desktop.

It consolidates the interaction ideas selected from Omarchy and related desktop research into one Vesper-native plan. Omarchy is an interaction reference only. Vesper must reuse its existing shell, Settings, input, audio, notification, sharing, AI and compositor infrastructure instead of creating parallel subsystems.

Current implementation must always be inspected before claiming any target below is complete.

## goals

The plan should make Vesper feel faster and more coherent in everyday use without turning it into a pile of one-off scripts.

Primary goals:

- one semantic action may have several input triggers, but only one implementation;
- common actions should be reachable without opening a full Settings page;
- persistent configuration belongs in Settings; transient actions belong in the shell/command layer;
- existing Caelestia and Vesper backends remain authoritative;
- local state should be event-driven where the owner exposes reliable events;
- every visible state must remain truthful when a backend is unavailable or stale;
- keyboard-first workflows should not remove mouse/touchpad workflows;
- runtime conveniences must not silently mutate declarative Nix configuration;
- new UI must follow Vesper visual standards rather than copying Omarchy styling.

## explicit non-goals

The following are intentionally excluded from this plan:

- Night Light integration;
- sensitive-content filtering for clipboard history;
- Dictation;
- a second notification daemon;
- a second clipboard database;
- a second AI quota/backend stack;
- a second sharing protocol or upload service;
- a second Settings application;
- a replacement for the existing Caelestia/Vesper shell;
- a duplicate workspace overview when existing Caelestia `showall` already satisfies the use case;
- QR-region decoding as part of this work;
- copying Omarchy colours, radii, spacing, chrome or visual identity.

## UI implementation contract

Omarchy and other desktops are interaction references only. They are not visual authorities for Vesper.

All new surfaces must use the active Vesper/Caelestia design system:

- Caelestia/Quickshell-native components and shared Vesper primitives;
- existing semantic colour, spacing, typography, radius and motion tokens;
- native Nexus settings rows for Settings surfaces;
- the active Vesper transient-surface visual language;
- `TOP-BAR-DOCK.md` when the planned top-bar/dock visual contract is active;
- reduced-motion, reduced-transparency and increased-contrast behavior where the owning surface supports it.

Rules:

- do not create an Omarchy-themed subsystem inside Vesper;
- do not invent a separate glass recipe for these features;
- avoid glass-on-glass nesting unless there is a semantic reason;
- prefer source-to-surface continuity and restrained movement over arbitrary fade-only transitions;
- transient surfaces must look related to the control that invoked them;
- indicators are views over authoritative state, not miniature independent state machines;
- QML renders normalized state and invokes bounded actions; it should not become the authoritative parser for system state.

## architecture and ownership

Use this ownership map when implementing the plan:

```text
Hyprland
  -> special workspaces
  -> focus/window/monitor geometry
  -> compositor zoom
  -> keybinding dispatch

Caelestia / Quickshell
  -> shell presentation
  -> launcher / command palette integration
  -> bar / OSD / capture presentation
  -> notification presentation
  -> transient UI geometry

Vesper Settings / Nexus extensions
  -> Default Agent
  -> Display arrangement/configuration
  -> Input / Assistant-key discoverability
  -> Shortcuts / conflict detection

Vesper backends
  -> structured state normalization
  -> safe settings persistence
  -> CWD attribution
  -> app identity / launch-or-focus decisions where needed
  -> display preview/revert transaction

PipeWire / WirePlumber
  -> audio device truth and default sink changes

LocalSend / OnionShare
  -> existing sharing transports

AI Hub / Agent Cockpit
  -> agent/provider/process status

Hermes
  -> recurring automation only, not quick reminders
```

Do not move ownership merely because a new UI surface needs to display the state.

---

# 1. Default Agent

Add **Default Agent** to:

```text
Settings
└── Apps
    └── Default Apps
        └── Default Agent
```

This is the one canonical selector for the generic Vesper "open my coding agent" intent.

Candidate values are derived from supported installed runtimes. Examples, only when installed/supported:

```text
Codex
Claude Code
OpenCode
Gemini CLI
GitHub Copilot CLI
Pi
other Vesper-supported agent runtimes
None
```

Rules:

- never hard-code one provider/runtime as the permanent default;
- `None` is a valid explicit state;
- this is an application/runtime default, not a provider/model preference;
- provider credentials and model/router policy remain under `Settings -> AI`;
- the generic launcher, physical Assistant/Copilot key and Quake Agent Console consume this same setting;
- there must not be separate defaults for the console, keyboard key and launcher;
- removing the selected runtime must produce an unavailable/None state, never silently launch another agent;
- persist through the normal Vesper settings/config ownership path;
- the row must use native Caelestia/Nexus UI.

Acceptance criteria:

- installed supported runtimes are discovered truthfully;
- changing Default Agent changes every generic agent launch path;
- `None` opens no fallback agent;
- uninstalling the selected agent cannot execute a different runtime by accident.

---

# 2. Quake Agent Console

Vesper should provide a Quake-style drop-down workspace for the Default Agent.

Primary triggers:

```text
physical Copilot / Assistant key   Toggle Agent Console
Super + `                          Toggle Agent Console
Super + Shift + `                  Move current window to Agent Console
```

The physical Copilot key means the dedicated assistant key present on newer laptop keyboards. It does **not** mean GitHub Copilot and does not imply GitHub Copilot CLI.

## input normalization

All triggers resolve to the same semantic action:

```text
Toggle Agent Console
```

Input handling should prefer the semantic assistant-key event exposed by the Linux input stack, such as `KEY_ASSISTANT` when available.

Some Copilot keyboards may expose a legacy compatibility chord such as Meta/Shift/F23. Support that only after validating the actual device/input event path. Do not globally hijack arbitrary F23 or Meta/Shift/F23 chords from unrelated keyboards.

Settings should expose the detected Assistant/Copilot key only when the hardware/input stack actually exposes one.

## compositor model

Use a Hyprland **special workspace**, not a fake floating terminal overlay.

Target behavior:

- drops from the top over the current workspace;
- current workspace stays in place underneath;
- underlying workspace may be lightly dimmed using Vesper visual rules;
- default height is roughly half of usable logical monitor space;
- calculate against monitor scale and reserved areas, not fixed physical pixels;
- recompute after monitor layout/scale/focus changes;
- remain tiled so two or more applications can sit side by side;
- hiding the console must not kill its applications;
- reopening reveals the same running session;
- when created empty, lazily launch Default Agent;
- do not launch an agent at login merely to seed the console;
- if Default Agent is `None` or unavailable, open an empty console truthfully;
- moving a second window into the console must tile rather than replace the agent;
- ordinary Hyprland window navigation should work where special-workspace semantics permit.

Conceptual flow:

```text
Copilot key / Super + `
          ↓
Toggle Agent Console
          ↓
special:agent-console
          ↓
empty?
 ├─ no  -> reveal existing tiled session
 └─ yes -> read Default Agent
            ├─ available -> launch lazily into special workspace
            └─ None/unavailable -> remain empty
```

Optional shell enhancement:

- a tiny Agent Cockpit status/header may be shown if it remains lightweight;
- it must not turn the console into a second AI dashboard;
- detailed quota/process views stay in Vesper Hub / Agent Cockpit.

Acceptance criteria:

- Copilot key and `Super + \`` toggle the same workspace;
- scale 1x, fractional scale and HiDPI all produce approximately half usable height;
- bar/reserved-area changes do not produce incorrect console size;
- two apps tile side by side in the console;
- closing/reopening preserves the session;
- no agent process is started before first use;
- `None` produces an empty console without an invented fallback.

---

# 3. Share: LocalSend + OnionShare

Expose one system-level **Share** action:

```text
Share
├── LocalSend
└── OnionShare
```

Semantics:

- LocalSend = fast local-network/device transfer;
- OnionShare = Tor/onion privacy-oriented sharing path;
- the UI must not imply identical privacy, reachability or lifetime semantics.

Entry points:

- global shortcut, candidate `Super + Ctrl + S` after conflict audit;
- command palette;
- file-manager context action where integration is reliable;
- screenshot/capture result actions where useful.

Possible later convenience actions may include Copy Path, Copy File URI or QR generation, but LocalSend and OnionShare remain the primary transports.

Do not implement another transfer backend merely to power this menu.

Acceptance criteria:

- both installed transports appear through one Share UX;
- absence of one transport does not fake availability;
- capture/file actions pass the intended file rather than launching an unrelated empty app;
- privacy wording stays transport-specific.

---

# 4. Stay Awake

Provide a temporary **Stay Awake** / caffeine action for inhibiting idle suspend/blanking.

Useful cases:

- downloads/transfers;
- builds;
- long-running agents;
- presentations;
- screen recording.

Requirements:

- use the existing system inhibitor mechanism;
- do not rewrite global power policy;
- show clear active/inactive state;
- show a compact shell indicator while active;
- disabling immediately releases the inhibitor;
- normal power/idle policy remains authoritative.

This is a runtime action, not a persistent Settings toggle.

---

# 5. Audio Output Cycle

Add a fast action to cycle currently usable PipeWire output devices.

Example:

```text
Speakers -> Headphones -> HDMI -> Bluetooth -> Speakers
```

Requirements:

- derive candidates from PipeWire/WirePlumber truth;
- skip disconnected/unavailable sinks;
- reuse the same backend used by normal audio controls;
- show the selected output through the normal Vesper OSD/notification language;
- preserve the full audio picker for direct non-linear selection.

A good hardware-media binding candidate is a modified mute/media key only after the shortcut audit confirms it does not conflict with current Vesper behavior.

---

# 6. Notification Actions and Replay

Extend the existing notification experience rather than replacing its daemon.

Target semantic actions:

```text
Dismiss latest
Dismiss all
Invoke latest actionable notification
Open notification history
Replay recent notifications
Toggle DND
```

Requirements:

- preserve application-provided actions when available;
- never invoke an expired/invalid action;
- DND suppresses presentation without deleting history;
- use one bounded source of truth for notification history;
- replay means re-present recent stored notifications, not fabricate new application events;
- shell restart must not resurrect dismissed notifications as new ones.

---

# 7. Quick Reminders

Provide a small reminder/timer flow without turning it into a calendar or Hermes job.

Example:

```text
Super + Ctrl + R
15m
pizza in oven
```

Requirements:

- accept supported duration or absolute-time syntax plus short message;
- use a reliable user-level timer mechanism;
- deliver through the normal notification system;
- persisted reminders survive shell restarts;
- expose active count/state when reminders exist;
- allow list/cancel/clear from the reminder surface/command palette;
- do not create a second general-purpose scheduler.

Hermes remains the only recurring research/automation scheduler.

---

# 8. Keyboard-first Smart Capture

Vesper's capture picker should be fully usable without a mouse.

Target interaction while the picker is active:

```text
Tab / Shift+Tab        cycle candidate windows
Arrow keys             move selection spatially
Enter                  capture highlighted target
Ctrl+Enter             capture focused/current monitor
Escape                 cancel
```

Requirements:

- highlighted target and captured rectangle use the same geometry source;
- use actual visible geometry rather than app-name heuristics;
- all enabled monitors remain selectable, including an empty secondary monitor;
- mouse selection remains available;
- extend the existing Caelestia capture path rather than build a second screenshot tool.

## shell/popup geometry

Quickshell/plugin panels may be implemented as monitor-sized transparent layer surfaces containing a smaller visible card.

When the shell can expose a trustworthy registered card geometry:

```text
capture target = visible card geometry
not = entire transparent layer surface
```

This applies to screenshot and screen-record selection where technically supported.

Acceptance criteria:

- keyboard and pointer paths select the same rectangles;
- moving through windows is deterministic;
- empty monitors are still targetable;
- visible plugin cards can be captured without a giant transparent bounding box;
- cancellation leaves no stale selection/input layer behind.

---

# 9. Capture Hub

Keep screenshot, region capture, screen recording, OCR and color picking under one coherent capture namespace.

Target actions:

```text
Screenshot
Region Screenshot
Record Screen
Record Region
OCR Region
Color Picker
```

Requirements:

- reuse current Caelestia/Vesper capture backends;
- Color Picker copies selected color in useful formats;
- recording has a visible active-state indicator;
- start/stop belongs to the same action family;
- results may expose `Share -> LocalSend / OnionShare`;
- do not add Dictation or Night Light.

---

# 10. CWD-aware Launch

When a terminal workflow has a trustworthy current directory, new terminal/file-manager actions should preserve it.

Example:

```text
active terminal cwd: ~/Code/vesper
New Terminal         -> ~/Code/vesper
Open Files Here      -> ~/Code/vesper
```

Requirements:

- resolve CWD from a trusted local process/window relationship;
- never infer it from terminal title text or a project label;
- if attribution fails, fall back to the normal default directory;
- terminal and file-manager launch share the same CWD resolver;
- avoid a second terminal profile just for this feature.

Suggested ownership:

```text
focused window
   ↓
stable terminal/app identity
   ↓
PID/process attribution
   ↓
resolved CWD
   ↓
terminal --dir / file-manager path
```

Acceptance criteria:

- active supported terminal opens a new terminal in the same directory;
- Files can open the same directory;
- non-terminal focus falls back safely;
- stale/dead PID cannot result in a fabricated path.

---

# 11. Display Arrange

`Settings -> Display` should include a native visual arrangement editor.

Target interaction:

- drag displays to arrange logical position;
- snap edges/corners when useful;
- preserve valid non-snapped layouts;
- identify similar displays clearly;
- select a display and change orientation/rotation;
- ordinary resolution, refresh rate, scale and mirror controls remain in the same Display surface;
- workspace-to-monitor assignment and saved profiles remain compatible with the same canonical monitor model.

## safe apply / automatic revert

Display changes require a confirmation transaction:

```text
proposed layout
      ↓
apply temporary runtime preview
      ↓
confirmation countdown
   ├─ Confirm -> keep runtime state / offer persistence path
   ├─ Reject  -> revert immediately
   └─ Timeout -> revert automatically
```

The countdown should default to a bounded window such as ~15 seconds unless Vesper's existing Settings transaction standard defines another value.

Requirements:

- keep the full previous layout snapshot until confirmation;
- a compositor/shell crash during the confirmation window must not leave an unsafe persistent configuration;
- persistence remains separate from temporary preview;
- persistent changes use the runtime-to-declarative bridge when structured Vesper config support exists;
- the visual editor must not become a second monitor database.

Acceptance criteria:

- rearranging two or more monitors works by drag;
- orientation changes are reflected in the preview;
- refusing or ignoring confirmation restores the exact prior runtime layout;
- a missing/disconnected monitor is handled truthfully;
- persistence never happens before explicit acceptance.

---

# 12. Context-aware Universal Actions

Vesper may expose semantic Copy/Paste/Cut actions that adapt to the focused application.

Goal:

```text
Copy
Paste
Cut
```

instead of forcing users to remember GUI vs terminal chords.

Requirements:

- terminal/TUI detection must use reliable application/window identity;
- terminal-hosted TUIs receive terminal-safe copy/paste behavior;
- do not send GUI `Ctrl+C` into a TUI where it can become SIGINT;
- normal GUI apps receive their standard toolkit shortcut;
- applications with incompatible input semantics may remain unsupported;
- do not steal current Vesper bindings without shortcut conflict analysis;
- semantic actions live in the same structured shortcut registry as other actions.

Implementation gate:

- audit current Caelestia/Vesper bindings first;
- specifically preserve existing behavior currently assigned to `Super+C` unless intentionally migrated through the canonical shortcut system.

---

# 13. Compositor Screen Zoom

Provide compositor-level zoom for inspecting small UI/accessibility use.

Target behavior:

- zoom around cursor/focus using Hyprland-native capabilities;
- repeated action increments zoom in controlled steps;
- separate action resets immediately to 100%;
- active zoom may expose an OSD/indicator;
- reduced-motion mode uses minimal transition;
- never confuse zoom with `Settings -> Display` scale.

Settings ownership:

- shortcuts are configurable under `Settings -> Shortcuts`;
- zoom amount is runtime state, not a persistent display-scale setting by default.

---

# 14. Bar Scroll + OSD

Volume and brightness bar controls should normalize high-resolution wheel/touchpad input into predictable steps.

Requirements:

- accumulate small touchpad wheel deltas;
- apply a real step only when the configured threshold/notch is crossed;
- use the same logical increment as existing volume/brightness controls unless a shared setting says otherwise;
- trigger OSD only when a real step is applied;
- mouse wheel and touchpad should feel consistent;
- reuse the existing volume/brightness backend and OSD;
- precise adjustment paths remain available.

Acceptance criteria:

- one mouse-wheel notch equals one logical step;
- a touchpad gesture does not produce dozens of tiny changes;
- OSD updates only for committed steps;
- no separate audio/brightness state exists in the bar widget.

---

# 15. Launch-or-Focus

Dedicated app actions may focus an existing application instead of spawning duplicates when that matches app semantics.

Rules:

- desktop-entry/application identity is authoritative;
- distinguish single-instance/focus-first apps from apps where multiple windows are expected;
- webapp/TUI wrappers need stable app IDs, not title regexes;
- inspect Caelestia launcher/dock behavior first;
- reuse existing correct launch-or-focus behavior rather than duplicate it;
- do not create another application identity registry.

Implementation should begin with apps that clearly benefit from focus-first semantics rather than applying it globally.

---

# 16. Workspace Layout Snapshots

Allow saving/restoring useful workspace arrangements.

Examples:

```text
Coding
  editor | agent | terminal

Research
  browser | Hermes

Communication
  browser | Vesktop
```

Requirements:

- save only compositor/app identity information that can be restored reliably;
- distinguish layout/placement from application-internal state;
- relaunch known apps only when explicitly supported;
- never claim to restore unsaved document/application state;
- handle monitor identity and missing monitors;
- a partial restore should report missing pieces rather than destroy the successful parts;
- snapshots live in one canonical registry;
- expose Save/Restore through the command palette and an appropriate Settings/desktop surface.

---

# 17. Unified Command Palette

Vesper should converge app launch, system actions and Settings navigation into one searchable command surface.

Target examples:

```text
Firefox
Wi-Fi
Share
OnionShare
Stay Awake
Audio Output
AI Usage
Default Agent
Agent Console
OCR
Record Screen
Reminder
Save Workspace Layout
Restore Workspace Layout
Privacy
Arrange Displays
Zoom
```

Rules:

- `Super + Space` remains the primary entry point once unified;
- application launch remains desktop-entry based;
- Settings results navigate to the real row rather than reimplement the setting inside the palette;
- privileged/destructive actions keep existing confirmation policy;
- aliases/keywords should live in a structured shared action registry;
- do not scatter duplicate search registries across QML files.

---

# 18. Keep-mapped Hidden Shell Surfaces

For expensive Quickshell layer surfaces that are frequently hidden/revealed, Vesper may keep the surface mapped and park it outside the visible region when measurement proves map/unmap rebuild is the performance problem.

This is an optimization pattern, not a requirement for every popup.

Requirements:

- hidden surface reserves no unintended exclusive zone;
- hidden surface intercepts no pointer/keyboard input;
- it must not appear in visible desktop captures;
- it must not leak onto adjacent monitors;
- background work continues only when that work was already intended to survive hiding;
- use only after measuring a meaningful improvement;
- state ownership remains with the original component.

Good candidates may include frequently toggled persistent shell surfaces such as the top bar, but implementation must be benchmark-driven.

---

# 19. Shell State Indicators

Show compact indicators only for temporary user-controlled states that are easy to forget.

Candidate states:

```text
Stay Awake
Recording
DND
active reminders
screen zoom
Tor/privacy state when already provided by Privacy HUD
active-agent count / quota pressure when already provided by AI Hub
```

Rules:

- indicator reads authoritative state;
- inactive low-priority indicators may disappear;
- clicking opens the owning control/status surface where useful;
- do not create new collectors just to draw an icon;
- Night Light and Dictation remain excluded.

---

# 20. Event-driven Local State

Reduce avoidable short-interval subprocess polling for local desktop state.

Prefer events, DBus/signals, sockets, compositor callbacks or long-lived subscriptions for:

- audio default-sink/device changes;
- network connectivity;
- monitor layout/focus;
- power/session idle state;
- notification state;
- recording state;
- agent process state when Agent Cockpit exposes it.

Bounded polling remains acceptable for inherently remote/periodic state such as provider quota refreshes.

The rule is not "no polling". The rule is: do not repeatedly spawn expensive local probes when the owner can push a trustworthy change.

---

# Settings integration

Do not create an "Omarchy features" page. Each persistent/discoverable control belongs to its real Settings owner.

Target information placement:

```text
Settings
├── Apps
│   └── Default Apps
│       └── Default Agent
│
├── Display
│   ├── Arrange Displays
│   ├── resolution / refresh / scale / orientation
│   ├── mirror / extend
│   ├── workspace assignment
│   └── saved monitor profiles
│
├── Input
│   └── Assistant / Copilot key
│       ├── detected hardware state
│       └── action: Toggle Agent Console
│
├── Shortcuts
│   ├── Toggle Agent Console
│   │   ├── physical Assistant/Copilot key when present
│   │   └── Super + `
│   ├── Move window to Agent Console
│   ├── Share
│   ├── Reminder
│   ├── Audio Output Cycle
│   ├── Zoom / Reset Zoom
│   ├── notification actions
│   └── universal semantic actions when implemented
│
└── Power & Performance
    └── normal idle/suspend policy remains authoritative
```

Settings rules:

- Default Agent is the only persistent agent selector for generic agent launch;
- physical Assistant/Copilot-key UI appears only when actually detected;
- its default semantic action is `Toggle Agent Console`;
- changing key behavior later must use the canonical shortcut registry/conflict checker;
- Display Arrange is part of Display, not a separate application;
- display confirmation/revert is a Settings transaction, not a hidden fire-and-forget script;
- CWD-aware launch is an ergonomic behavior, not a toggle by default;
- bar-scroll normalization is implementation behavior, not a Settings toggle by default;
- Launch-or-Focus is app-action behavior, not a global toggle until a real policy need appears;
- Capture keyboard navigation belongs to Capture help/shortcut discovery, not a standalone page;
- compositor zoom belongs in Shortcuts, not persistent Display scale;
- Stay Awake is temporary runtime state, not a replacement for power policy;
- LocalSend/OnionShare retain their existing app/service configuration ownership.

`APPS-SETTINGS.md` remains authoritative for the Default Apps/installed-app surface. `SETTINGS.md` remains authoritative for the wider Settings information architecture.

---

# shortcut plan

Every final chord must pass the shared Vesper shortcut conflict audit.

Conceptual reserved actions:

```text
physical Copilot / Assistant key   Toggle Agent Console
Super + `                          Toggle Agent Console
Super + Shift + `                  Move window to Agent Console
Super + Ctrl + S                   Share
Super + Ctrl + R                   Quick Reminder
```

Other actions such as Stay Awake, Audio Output Cycle, notification controls, zoom and universal Copy/Paste must be assigned only after checking current Vesper/Caelestia bindings.

Do not change an existing binding solely to mimic Omarchy.

The shortcut registry should expose semantic action IDs so hardware keys, user chords, command-palette actions and Settings all point at the same action.

Example conceptual IDs:

```text
agent.console.toggle
agent.console.move-current-window
share.open
reminder.create
stay-awake.toggle
audio.output.next
capture.open
capture.ocr
capture.color-picker
display.zoom.in
display.zoom.reset
notification.dismiss-latest
notification.dismiss-all
notification.invoke-latest
notification.history
notification.dnd.toggle
```

---

# implementation phases

## Phase 0 — audit and shared foundations

Before adding features:

1. inventory current Caelestia/Vesper shell actions and hotkeys;
2. inventory current app identity/launch behavior;
3. inventory capture backend and popup geometry ownership;
4. inventory notification history/actions/DND support;
5. inventory PipeWire audio APIs already exposed to QML/backend;
6. inventory Settings row/navigation primitives;
7. identify the canonical structured shortcut/action registry or create one if missing;
8. identify which local-state pollers can be replaced with existing events;
9. document conflicts instead of silently replacing working behavior.

Exit criteria:

- each feature has an owner and no duplicate backend is planned;
- final shortcut candidates are conflict-checked;
- Settings navigation targets are known;
- existing functionality that already satisfies a target is marked reuse, not rebuilt.

## Phase 1 — agent-console foundation

Implement together:

1. Default Agent canonical setting;
2. supported-agent runtime discovery;
3. physical Assistant/Copilot-key normalization;
4. semantic `agent.console.toggle` action;
5. Hyprland special workspace;
6. lazy Default Agent seeding;
7. half-height scale-aware geometry;
8. multi-window tiling;
9. Vesper-native presentation/accessibility behavior.

Verify on:

- normal keyboard without Assistant key;
- hardware exposing semantic assistant key when available;
- compatibility Copilot-key event path when available;
- 1x scale;
- fractional scale;
- HiDPI;
- one and multiple monitors.

## Phase 2 — everyday utility actions

Implement:

1. Share menu with LocalSend + OnionShare;
2. Stay Awake;
3. Audio Output Cycle;
4. notification actions/replay;
5. Quick Reminders;
6. matching temporary shell indicators.

Goal: useful daily ergonomics without changing major shell architecture.

## Phase 3 — capture and launch ergonomics

Implement:

1. Keyboard-first Smart Capture;
2. shell/plugin visible-card geometry targets;
3. Color Picker integration;
4. Capture result Share actions;
5. CWD-aware terminal launch;
6. CWD-aware file-manager launch;
7. Launch-or-Focus audit and first safe app actions.

## Phase 4 — display and input polish

Implement:

1. Display Arrange native editor;
2. safe temporary apply;
3. confirmation countdown and exact revert;
4. persistent runtime-to-declarative handoff when supported;
5. compositor screen zoom;
6. zoom indicator/OSD;
7. bar scroll delta normalization and OSD consistency.

## Phase 5 — command and workflow convergence

Implement:

1. unified structured action registry across shell/Settings;
2. unified command palette;
3. semantic Settings navigation results;
4. workspace layout snapshots;
5. context-aware universal actions after binding/app identity audit.

## Phase 6 — performance and event cleanup

After user-facing behavior is stable:

1. profile Quickshell show/hide/map/unmap costs;
2. apply keep-mapped parking only where measurement supports it;
3. replace avoidable local subprocess polling with owner events/signals;
4. verify multi-monitor and capture behavior after optimization;
5. remove superseded helper scripts/dead code only after proving no callers remain.

---

# testing and acceptance matrix

Every implementation PR should test the behavior it introduces rather than only syntax/build success.

## shell/input

Test:

- shortcut conflicts;
- action-ID routing;
- physical Assistant key absent/present states;
- no accidental F23 hijacking;
- special-workspace toggle idempotency;
- multi-window console tiling;
- console persistence across hide/show.

## scaling and monitors

Test:

- 1.0 scale;
- common fractional scale;
- 2.0 scale;
- reserved top bar area;
- focus moving between differently scaled monitors;
- monitor connect/disconnect;
- Display Arrange auto-revert;
- no unsafe persistence before confirmation.

## capture

Test:

- mouse selection;
- Tab navigation;
- arrow navigation;
- Enter/Ctrl+Enter;
- empty secondary monitor;
- shell/plugin visible-card capture;
- cancellation cleanup;
- screenshot and recording target parity where both are supported.

## audio/OSD

Test:

- mouse wheel;
- high-resolution touchpad scroll;
- disconnected sink skipped;
- output cycle ordering remains stable enough to understand;
- OSD reflects the actual selected sink/value;
- no duplicate backend state.

## reminders/notifications

Test:

- shell restart;
- reminder persistence/cancel;
- DND history behavior;
- dismiss latest/all;
- actionable notification validity;
- bounded history;
- replay does not manufacture new app events.

## launch behavior

Test:

- supported terminal CWD;
- unknown terminal fallback;
- dead/stale PID fallback;
- file manager opens same resolved directory;
- Launch-or-Focus does not collapse apps that legitimately need multiple windows.

## UI/accessibility

Test new surfaces with:

- bright wallpaper/background;
- dark wallpaper/background;
- high-frequency/mixed background;
- reduced motion;
- reduced transparency where applicable;
- increased contrast;
- keyboard-only operation for the flows that claim it.

---

# failure and fallback rules

Use truthful degradation throughout the plan.

Examples:

```text
Default Agent missing
  -> console opens empty / setting shows unavailable

Assistant key not detected
  -> no fake hardware row; Super+` still works

PipeWire sink disappears
  -> skip it; do not select stale device

CWD cannot be attributed
  -> normal launch directory

Display confirmation times out
  -> exact prior layout restored

OnionShare not installed
  -> unavailable/hidden according to normal action policy

notification action expired
  -> do not invoke

plugin capture geometry unavailable
  -> use the safe existing capture target; never invent coordinates
```

Unknown must remain unknown. Failure in an enhancement must not silently trigger an unrelated action.

---

# persistence rules

Separate runtime state from persistent declarative configuration.

Runtime-only examples:

- Agent Console visible/hidden;
- Stay Awake;
- current compositor zoom;
- current audio output;
- notification DND runtime state according to its existing owner;
- temporary display preview before confirmation.

Persistent examples when supported:

- Default Agent;
- shortcut customization;
- accepted monitor layout/profile through the structured Vesper configuration path;
- saved workspace layout snapshots.

No QML surface should silently write arbitrary Nix/Lua text. Persistent system changes use the guarded Vesper runtime-to-declarative contract.

---

# documentation maintenance

When a feature lands:

1. change its section from target language to documented current behavior where appropriate;
2. record the canonical backend/source of truth;
3. remove implementation alternatives that are no longer relevant;
4. update shortcut/help discovery from the shared registry rather than maintaining a second manual list;
5. delete superseded/dead helpers only after callers are verified absent;
6. keep this document as the one canonical desktop-ergonomics plan instead of creating per-feature markdown files.

## final implementation order

The preferred order is:

```text
0  Audit / action registry / ownership
1  Default Agent
2  Physical Copilot/Assistant key normalization
3  Quake Agent Console
4  Share: LocalSend + OnionShare
5  Stay Awake
6  Audio Output Cycle
7  Notification actions/replay
8  Quick Reminders
9  Keyboard-first Smart Capture
10 Capture visible plugin/card geometry
11 Color Picker / Capture Hub cleanup
12 CWD-aware terminal + Files launch
13 Display Arrange + automatic revert
14 Compositor screen zoom
15 Bar scroll normalization + OSD
16 Launch-or-Focus
17 Unified command palette/action registry convergence
18 Workspace layout snapshots
19 Context-aware universal actions
20 Shell state indicator polish
21 Event-driven cleanup
22 Keep-mapped shell-surface optimizations where benchmarked
```

The ordering is intentionally dependency-aware: establish canonical state/actions first, ship low-risk utility wins next, then larger display/input/workflow changes, and optimize only after behavior is correct.
