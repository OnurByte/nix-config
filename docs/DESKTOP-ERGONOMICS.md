# Desktop Ergonomics

Status: **spec**

This document defines the small, high-frequency desktop workflows Vesper should provide on top of the existing Caelestia + Hyprland desktop.

The goal is not to copy another desktop shell or duplicate capabilities Vesper already has. Vesper should reuse its current Caelestia, Hyprland, PipeWire, LocalSend, OnionShare, AI Hub, Agent Cockpit and notification infrastructure and add a thin ergonomic layer around them.

Current implementation must be inspected before claiming any target behavior below is already complete.

## product principles

- prefer one coherent entry point over separate one-off scripts;
- reuse authoritative Vesper/Caelestia backends instead of parsing the same state again in QML;
- prefer compositor/system events for local state and bounded polling only where the upstream source is inherently poll-based;
- preserve truthful unknown/stale states instead of inventing values;
- keep hotkeys discoverable through the shared shortcut/command registry;
- do not create a second settings application, second notification daemon, second clipboard database, second AI quota backend or second sharing stack;
- runtime convenience must not silently mutate declarative Nix configuration.

Three ideas are intentionally out of scope for this plan:

- Night Light integration is not part of the Vesper desktop ergonomics target;
- sensitive-content filtering for clipboard history is not part of this target;
- Dictation is not part of the Vesper desktop ergonomics target.

## UI implementation contract

Omarchy and other desktops are interaction references only. They are not visual authorities for Vesper.

All new surfaces in this document must use the active Vesper/Caelestia design system:

- Caelestia/Quickshell-native components and shared Vesper primitives;
- existing semantic colour, spacing, typography, radius and motion tokens;
- the active Vesper visual authority for transient shell surfaces;
- `TOP-BAR-DOCK.md` when the planned top-bar/dock visual contract is active;
- existing accessibility behavior such as reduced motion, reduced transparency and increased contrast where the owning surface supports it.

Rules:

- do not copy Omarchy colours, radii, spacing, shadows or panel chrome merely because an interaction originated there;
- do not create a new glass recipe for these features;
- do not nest decorative glass cards inside a transient glass surface without a semantic reason;
- use source-to-surface continuity and restrained motion where practical rather than arbitrary fade-only animation;
- settings rows must look and behave like native Caelestia/Nexus settings rows;
- shell controls must use the same state/feedback language as existing Vesper shell controls.

## Quake Agent Console

Vesper should provide a Quake-style drop-down workspace for the default coding agent.

Primary triggers:

```text
physical Copilot / Assistant key   toggle Agent Console
Super + `                          toggle Agent Console
Super + Shift + `                  move the current window to Agent Console
```

The physical Copilot key means the dedicated assistant key present on newer laptop keyboards. It does **not** mean GitHub Copilot and it does not select the GitHub Copilot CLI.

Input handling must normalize the real hardware event into the semantic `Toggle Agent Console` action. Prefer a semantic assistant-key event such as Linux `KEY_ASSISTANT` / the corresponding input-stack symbol when exposed. On hardware that instead reports the legacy Copilot chord such as Meta/Shift/F23, recognize it only through the input/keybinding layer after validating the actual device event. Do not globally hijack arbitrary F23 or Meta/Shift/F23 input from unrelated keyboards merely to emulate a Copilot key.

Both the physical Copilot key and `Super + \`` must invoke the same canonical action and state. There must not be a second agent console implementation for the hardware key.

The console should be implemented as a Hyprland special workspace rather than a fake floating terminal overlay.

Target behavior:

- it drops down from the top over the current workspace;
- the workspace beneath remains in place and may be visually dimmed while the console is open;
- it covers roughly half of the usable monitor height by default;
- sizing is scale-aware and uses usable logical monitor space rather than a fixed pixel height;
- monitor scale/layout changes recompute the console size;
- the console remains tiled so two or more applications can sit side by side inside it;
- closing/toggling the console does not destroy its running applications;
- if the console is created empty, launch the configured **Default Agent** lazily rather than starting an agent at login;
- if no Default Agent is selected or the selected runtime is unavailable, open the console without fabricating a fallback agent;
- moving another window into the console must not replace or kill the existing agent;
- ordinary workspace/window bindings should continue to work inside the special workspace where Hyprland semantics allow it.

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
 └─ yes -> launch Apps -> Default Apps -> Default Agent
```

Example:

```text
┌───────────────────────┬───────────────────────┐
│ Codex / Claude / ...  │ Ghostty / logs / app │
│                       │                       │
└───────────────────────┴───────────────────────┘
│             current workspace below          │
```

The console may surface a tiny Agent Cockpit header/status when that can be done without turning the scratchpad into a second AI dashboard. Detailed process state remains owned by Agent Cockpit / Vesper Hub.

## Default Agent

Add **Default Agent** to:

```text
Settings
└── Apps
    └── Default Apps
        └── Default Agent
```

This belongs beside other user-level defaults because it selects which installed application/runtime handles the generic "open my coding agent" intent.

Target choices should be populated only from supported installed runtimes, for example when present:

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

- do not hard-code a provider as the default;
- `None` is a valid explicit state;
- selecting Default Agent does not alter provider credentials, model policy or per-agent capability policy;
- AI provider/model configuration remains under `Settings -> AI`;
- the generic agent launcher, physical Copilot key and Quake Agent Console consume this same canonical Default Agent selection;
- do not keep separate defaults for the launcher, Copilot key, Agent Console and AI settings;
- if an agent is removed, the stale default must become unavailable/None rather than executing an unrelated fallback;
- persist the selection through the Vesper settings/config ownership path rather than a hidden mutable dotfile when a declarative setting is available.

## Stay Awake

Provide a one-action **Stay Awake** / caffeine toggle for temporarily inhibiting idle suspend/blanking while the user needs the machine to remain active.

Useful cases include:

- long downloads or transfers;
- builds and local jobs;
- long-running coding/research agents;
- presentations;
- screen recording.

Requirements:

- use the existing system inhibitor mechanism rather than disabling global power policy permanently;
- expose clear active/inactive state;
- make the active state visible in the shell so it is difficult to leave enabled accidentally;
- turning it off releases the inhibitor immediately;
- Settings power policy remains authoritative for normal idle/suspend behavior.

## Audio Output Cycle

Add a fast action for cycling among currently usable PipeWire output devices without opening the full audio panel.

Example:

```text
Speakers -> Headphones -> HDMI -> Bluetooth -> Speakers
```

Rules:

- derive the candidates from the current PipeWire/WirePlumber state;
- skip unavailable/disconnected sinks;
- changing the default output must use the same audio backend as the normal shell controls;
- show a short OSD/notification naming the selected output;
- keep the full audio picker available for non-linear selection.

## Share

Vesper should expose one system-level **Share** action rather than treating LocalSend and OnionShare as unrelated applications.

Default flow:

```text
Share
├── LocalSend
└── OnionShare
```

Semantics:

- **LocalSend** is the fast local-network/device-to-device path;
- **OnionShare** is the Tor/onion privacy-oriented sharing path;
- the menu must not imply these two transports have the same privacy, reachability or lifetime semantics.

Target entry points:

- global shortcut such as `Super + Ctrl + S`;
- command palette;
- file-manager context action where integration is reliable;
- screenshot/capture result actions where useful.

Possible later convenience actions may include Copy Path, Copy File URI or QR, but LocalSend and OnionShare are the primary Vesper sharing transports.

Do not implement another transfer protocol or upload service merely to power this menu.

## Notification Actions and Replay

Extend the existing notification experience with high-frequency actions instead of replacing the notification daemon.

Target actions:

```text
dismiss last
dismiss all
invoke last actionable notification
open notification history
replay recent notifications
DND toggle
```

Replay means re-presenting recent stored notifications through the existing shell presentation, not fabricating new application events.

Requirements:

- preserve application-provided actions when the notification backend exposes them;
- do not invoke an action if it is no longer valid;
- DND suppresses presentation according to the existing notification policy without deleting history;
- recent history should remain bounded;
- notification history and replay should use one source of truth rather than two independent logs;
- shell restart/reload should not silently misrepresent already-dismissed notifications as new events.

## Quick Reminders

Provide a lightweight reminder path for short-lived timers without requiring a full calendar/task application.

Example:

```text
Super + Ctrl + R
15m
pizza in oven
```

Target behavior:

- accept a duration or supported absolute time plus a short message;
- schedule through a reliable user-level timer mechanism;
- deliver through the normal notification system;
- expose active reminder count/state in the shell when reminders exist;
- support cancel/list from the command palette or reminder surface;
- persisted reminders should survive shell restarts;
- do not create a second general-purpose Hermes scheduler for this feature.

Hermes remains the owner of recurring research/automation jobs. Quick Reminders are simple user timers.

## Workspace Layout Snapshots

Allow users to save and restore useful workspace window arrangements.

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

- save only information the compositor can reliably identify and restore;
- distinguish layout/placement from application state;
- restoration may relaunch known applications when explicitly supported, but must not pretend it can restore arbitrary unsaved application contents;
- multi-monitor mappings must account for monitor identity/availability;
- a missing application or monitor should produce a partial/diagnostic restore rather than destroying the rest of the layout;
- expose Save Layout and Restore Layout through the command palette and an appropriate Settings/desktop surface;
- keep snapshots user-editable/removable through one canonical registry.

## Unified Command Palette

Vesper should converge launcher, system actions and searchable Settings actions into one command surface instead of growing parallel launchers.

Target queries/actions include:

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
```

The palette should search a structured registry shared with Settings/shortcut discovery where possible.

Rules:

- application launch remains desktop-entry based;
- privileged/destructive actions keep their normal confirmation rules;
- Settings results navigate to the real setting rather than reproducing the setting inside the palette;
- action aliases/keywords should not become a second hand-maintained registry scattered across QML files;
- `Super + Space` should remain the primary user-facing entry point once the surfaces are unified.

## Capture Hub

Vesper already has screenshot, region capture, screen recording, OCR, clipboard and emoji capabilities. Do not reimplement them.

Present the capture family through one coherent menu/command namespace:

```text
Screenshot
Region Screenshot
Record Screen
Record Region
OCR Region
Color Picker
```

Target addition:

- **Color Picker** copies a selected screen color in useful formats.

Requirements:

- reuse Caelestia capture backends where they already own screenshot/record behavior;
- recording must expose a visible active-state indicator;
- stopping a capture must be available from the same global action family;
- capture results may expose Share -> LocalSend / OnionShare where that is useful;
- do not add Night Light or Dictation to the capture or utility family.

## Keyboard-first Smart Capture

Capture selection should be fully usable without a mouse.

When the region/window picker is active, target interactions include:

```text
Tab / Shift+Tab        cycle candidate windows
Arrow keys             move selection spatially
Enter                  capture highlighted target
Ctrl+Enter             capture focused/current monitor
Escape                 cancel
```

Requirements:

- use the actual visible target geometry rather than guessing from application names;
- the highlighted target and the captured rectangle must be the same geometry source;
- when a Vesper/Caelestia plugin panel or popout is visually a card inside a larger transparent layer-shell surface, capture the visible registered card geometry rather than the entire transparent monitor-sized layer whenever the shell can expose that geometry reliably;
- every enabled monitor must remain selectable even when it has no ordinary window;
- mouse selection remains available; keyboard-first behavior is additive;
- avoid a second screenshot picker implementation if the current Caelestia capture surface can be extended.

## CWD-aware Launch

Launching a new terminal or file manager from a terminal-focused workflow should preserve the useful working directory when it can be determined truthfully.

Target behavior:

```text
active terminal cwd: ~/Code/vesper
New Terminal         -> ~/Code/vesper
Open Files Here      -> ~/Code/vesper
```

Rules:

- obtain CWD from a trusted local process/window relationship rather than parsing terminal title text;
- if the active window is not a supported terminal or its CWD cannot be attributed safely, fall back to the normal default directory;
- never fabricate a directory from a project label or window title;
- generic terminal launch and file-manager launch should share the same CWD-resolution backend;
- this is default ergonomic behavior and should not require a second terminal profile.

## Display Arrange

`Settings -> Display` should include a native visual arrangement surface for connected displays.

Target interaction:

- drag displays to arrange their logical position;
- snap edges/corners where useful without preventing free valid layouts;
- select a display and rotate/change orientation through the same native surface;
- show enough identity information to distinguish similar monitors;
- apply through the same display backend used by the ordinary Display controls.

Safety contract:

```text
Apply layout
    ↓
preview becomes live
    ↓
confirmation countdown
    ├─ Confirm -> keep
    └─ timeout / reject -> revert automatically
```

A bad layout must not strand the user on an unusable output arrangement. Runtime preview changes should revert automatically after a bounded confirmation window unless explicitly accepted.

Persistent layout changes remain subject to Vesper's runtime-to-declarative Settings contract. The visual editor must not become a second monitor configuration database.

## Context-aware Universal Actions

Vesper may provide universal Copy/Paste-style actions that adapt to the focused application class/capability, but only after auditing existing Caelestia bindings and avoiding conflicts.

The goal is semantic actions such as:

```text
Copy
Paste
Cut
```

rather than teaching the user different chords for terminal, TUI and GUI contexts.

Requirements:

- terminal/TUI detection must come from reliable app/window identity, not broad title regexes;
- a terminal-hosted TUI must receive terminal-safe copy/paste chords rather than a GUI chord that could send SIGINT or literal input;
- ordinary GUI applications receive their normal toolkit shortcuts;
- do not claim a universal action for applications whose input model cannot be safely adapted;
- do not steal the current `Super + C` or any other existing Vesper chord without the canonical shortcut-conflict check;
- expose the resulting semantic actions through the same structured shortcut registry as other Vesper actions.

## Compositor Screen Zoom

Provide a lightweight Hyprland/compositor-level screen zoom action for inspecting small UI and accessibility use without launching another application.

Target behavior:

- zoom around the cursor/focus point using compositor-native capabilities;
- repeated action may increment zoom in controlled steps;
- provide a direct reset-to-100% action;
- make active zoom state discoverable through OSD/indicator feedback where useful;
- reduced-motion policy should avoid unnecessary animated zoom transitions;
- do not confuse compositor zoom with per-display scale in `Settings -> Display`.

## Bar Scroll + OSD Interaction

Volume and brightness controls in the bar should normalize mouse-wheel and touchpad scrolling into predictable steps and show the same feedback language as keyboard media keys.

Requirements:

- accumulate high-resolution touchpad wheel deltas instead of applying an action for every tiny event;
- map completed notches/thresholds to consistent steps such as the existing Vesper volume/brightness increment;
- trigger the normal Vesper OSD only when a real step is applied;
- keep precise adjustment paths available where Vesper already exposes them;
- do not add a second volume/brightness backend for bar scrolling.

## Launch-or-Focus

Dedicated application actions may focus an existing matching application window instead of spawning duplicates when that matches the intended app semantics.

Rules:

- desktop-entry/application identity remains authoritative;
- distinguish apps where multiple independent windows are expected from single-instance or focus-first apps;
- web-app and terminal/TUI wrappers must use stable application identity rather than window titles;
- inspect existing Caelestia launcher/dock behavior first and reuse it when it already provides correct launch-or-focus semantics;
- do not introduce a parallel application identity registry.

## Keep-mapped Hidden Shell Surfaces

For expensive Quickshell layer surfaces that are frequently hidden/revealed, implementation may keep the surface mapped and move/park it out of the visible region when this measurably avoids scene-graph rebuild cost and does not break compositor behavior.

This is an optimization contract, not a requirement to keep every hidden popup alive.

Requirements:

- hidden surfaces must reserve no unintended exclusive zone;
- they must not intercept pointer/keyboard input;
- they must not appear in captures of the visible desktop;
- multi-monitor parking must not leak visible content onto adjacent outputs;
- background work should continue only if it was already intended to survive hiding;
- prefer this technique only after measurement shows map/unmap churn is the problem.

## Shell State Indicators

The bar/shell should show compact temporary indicators for user-controlled states that are easy to forget.

Useful states include:

```text
Stay Awake
Recording
DND
active reminders
Tor/privacy state when already provided by Privacy HUD
active-agent count / quota pressure when already provided by AI Hub
screen zoom when active and useful
```

Rules:

- indicators are views over authoritative state, not independent toggles with their own state files;
- keep them compact and hide inactive low-priority indicators where appropriate;
- clicking an indicator should open the owning control/status surface when useful;
- Night Light and Dictation are intentionally not included;
- do not duplicate Privacy HUD or AI Hub data collectors merely to render an icon.

## Event-Driven Local State

Vesper should reduce avoidable short-interval subprocess polling for local desktop state.

Prefer compositor/system events, DBus/signals, sockets or long-lived backend subscriptions for state such as:

- audio device/default-sink changes;
- network connectivity;
- monitor layout/focus changes;
- power/session idle state;
- notification state;
- recording state;
- live agent process events when Agent Cockpit can provide them.

Bounded polling remains acceptable for inherently remote or periodic data such as provider quota refreshes.

The rule is not "no polling". The rule is: do not repeatedly spawn expensive local probes when the owner can push a trustworthy state change.

## Settings integration

The ergonomics in this document should appear in Settings only where persistent configuration or discoverability is useful. Do not create a generic "Omarchy features" page.

Target ownership:

```text
Settings
├── Apps
│   └── Default Apps
│       └── Default Agent
│
├── Display
│   ├── Arrange Displays
│   └── ordinary resolution / scale / orientation / mirror controls
│
├── Input
│   └── Assistant / Copilot key
│       └── action: Toggle Agent Console
│
├── Shortcuts
│   ├── Toggle Agent Console
│   │   ├── physical Assistant/Copilot key when present
│   │   └── Super + `
│   ├── Move window to Agent Console
│   ├── Share
│   ├── Reminder
│   ├── Screen Zoom / Reset Zoom
│   └── universal semantic actions when implemented
│
└── Power & Performance
    └── normal idle/suspend policy remains authoritative
```

Settings rules:

- **Default Agent** is the only persistent selector that determines what the generic agent console launches;
- the physical Copilot/Assistant key row should report the detected hardware capability truthfully and must not appear as a fake device-specific control on hardware that does not expose such a key;
- the default Copilot/Assistant-key action is `Toggle Agent Console`;
- if shortcut customization permits changing that action later, it must go through the canonical shortcut registry and conflict detection;
- `Arrange Displays` is part of Display, not a standalone app;
- the display confirmation/revert countdown belongs to the Display transaction UX, not a hidden script;
- CWD-aware launch, bar-scroll normalization and launch-or-focus are interaction behaviors, not settings toggles by default;
- Capture keyboard navigation belongs to Capture behavior and shortcut help, not a new settings page;
- compositor zoom shortcuts belong in Shortcuts; do not expose zoom level as persistent display scale;
- Stay Awake is a temporary runtime action, not a persistent replacement for idle/suspend settings;
- LocalSend and OnionShare configuration remain owned by their existing app/service surfaces; Settings should not create a third sharing configuration stack.

## shortcut family

Final chords must pass the shared Vesper shortcut conflict check. Conceptually reserve a coherent family such as:

```text
Copilot / Assistant key  Agent Console
Super + `                Agent Console
Super + Shift + `        Move window to Agent Console
Super + Ctrl + S         Share
Super + Ctrl + R         Quick Reminder
```

Stay Awake, audio-output cycle, notification actions, zoom, universal actions and capture actions should be assigned through the canonical shortcut registry after conflicts with existing Vesper/Caelestia bindings are checked.

Do not silently replace existing bindings solely to mimic Omarchy.

## implementation order

Recommended order:

1. Default Agent canonical setting;
2. physical Copilot/Assistant-key normalization;
3. Quake Agent Console consuming Default Agent;
4. UI-token/visual-authority wiring for the console and new transient surfaces;
5. Share menu with LocalSend + OnionShare;
6. Stay Awake;
7. audio-output cycle;
8. notification actions/replay;
9. Quick Reminders;
10. Keyboard-first Smart Capture;
11. CWD-aware terminal/file-manager launch;
12. Display Arrange with automatic confirmation/revert;
13. compositor screen zoom;
14. bar scroll + OSD normalization;
15. unified command palette registry;
16. workspace layout snapshots;
17. Capture Hub addition: Color Picker;
18. launch-or-focus audit/integration;
19. context-aware universal actions after shortcut/application audit;
20. shell state indicators;
21. keep-mapped optimization where measurement justifies it;
22. event-driven cleanup of local state sources.

## implementation rules

When implementing this document:

1. inspect the current Hyprland/Caelestia behavior first;
2. reuse existing Vesper services and QML surfaces instead of forking equivalent infrastructure;
3. keep Default Agent canonical and shared by all generic agent launch paths, including the physical Copilot/Assistant key;
4. normalize real assistant-key hardware events into the semantic Agent Console action rather than assuming every laptop emits one fixed chord;
5. keep Quake Agent Console as a tiled special workspace, not a fixed floating terminal;
6. obey the Vesper UI implementation contract and active visual authority rather than copying Omarchy presentation;
7. keep LocalSend and OnionShare behind one Share UX without merging their transport semantics;
8. keep quick reminders separate from Hermes recurring automation;
9. preserve Settings/action safety rules for destructive or declarative mutations;
10. use automatic revert for risky display-layout previews;
11. prefer event-driven local state where the owner exposes it;
12. audit current Caelestia behavior before adding launch-or-focus or universal shortcut logic;
13. do not add Night Light as part of this work;
14. do not add sensitive clipboard filtering as part of this work;
15. do not add Dictation as part of this work;
16. update this document's status/current-state notes as features become implemented.
