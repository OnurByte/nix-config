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

Two ideas are intentionally out of scope for this plan:

- Night Light integration is not part of the Vesper desktop ergonomics target;
- sensitive-content filtering for clipboard history is not part of this target.

## Quake Agent Console

Vesper should provide a Quake-style drop-down workspace for the default coding agent.

Default bindings:

```text
Super + `          toggle Agent Console
Super + Shift + `  move the current window to Agent Console
```

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
Super + `
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
- the generic agent launcher and Quake Agent Console consume this same canonical Default Agent selection;
- do not keep separate defaults for the launcher, Agent Console and AI settings;
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
Dictation
```

Target additions:

- **Color Picker** copies a selected screen color in useful formats;
- **Dictation** records/transcribes speech through the selected supported local/provider path and returns text to the clipboard/current input workflow.

Requirements:

- reuse Caelestia capture backends where they already own screenshot/record behavior;
- recording/dictation must expose visible active-state indicators;
- stopping a capture must be available from the same global action family;
- capture results may expose Share -> LocalSend / OnionShare where that is useful;
- do not add Night Light to the capture or utility family.

## Shell State Indicators

The bar/shell should show compact temporary indicators for user-controlled states that are easy to forget.

Useful states include:

```text
Stay Awake
Recording
Dictation
DND
active reminders
Tor/privacy state when already provided by Privacy HUD
active-agent count / quota pressure when already provided by AI Hub
```

Rules:

- indicators are views over authoritative state, not independent toggles with their own state files;
- keep them compact and hide inactive low-priority indicators where appropriate;
- clicking an indicator should open the owning control/status surface when useful;
- Night Light is intentionally not included;
- do not duplicate Privacy HUD or AI Hub data collectors merely to render an icon.

## Event-Driven Local State

Vesper should reduce avoidable short-interval subprocess polling for local desktop state.

Prefer compositor/system events, DBus/signals, sockets or long-lived backend subscriptions for state such as:

- audio device/default-sink changes;
- network connectivity;
- monitor layout/focus changes;
- power/session idle state;
- notification state;
- recording/dictation state;
- live agent process events when Agent Cockpit can provide them.

Bounded polling remains acceptable for inherently remote or periodic data such as provider quota refreshes.

The rule is not "no polling". The rule is: do not repeatedly spawn expensive local probes when the owner can push a trustworthy state change.

## shortcut family

Final chords must pass the shared Vesper shortcut conflict check. Conceptually reserve a coherent family such as:

```text
Super + `              Agent Console
Super + Shift + `      Move window to Agent Console
Super + Ctrl + S       Share
Super + Ctrl + R       Quick Reminder
```

Stay Awake, audio-output cycle, notification actions and capture actions should be assigned through the canonical shortcut registry after conflicts with existing Vesper/Caelestia bindings are checked.

Do not silently replace existing bindings solely to mimic Omarchy.

## implementation order

Recommended order:

1. Default Agent canonical setting;
2. Quake Agent Console consuming Default Agent;
3. Share menu with LocalSend + OnionShare;
4. Stay Awake;
5. audio-output cycle;
6. notification actions/replay;
7. Quick Reminders;
8. unified command palette registry;
9. workspace layout snapshots;
10. Capture Hub additions: Color Picker + Dictation;
11. shell state indicators;
12. event-driven cleanup of local state sources.

## implementation rules

When implementing this document:

1. inspect the current Hyprland/Caelestia behavior first;
2. reuse existing Vesper services and QML surfaces instead of forking equivalent infrastructure;
3. keep Default Agent canonical and shared by all generic agent launch paths;
4. keep Quake Agent Console as a tiled special workspace, not a fixed floating terminal;
5. keep LocalSend and OnionShare behind one Share UX without merging their transport semantics;
6. keep quick reminders separate from Hermes recurring automation;
7. preserve Settings/action safety rules for destructive or declarative mutations;
8. prefer event-driven local state where the owner exposes it;
9. do not add Night Light as part of this work;
10. do not add sensitive clipboard filtering as part of this work;
11. update this document's status/current-state notes as features become implemented.
