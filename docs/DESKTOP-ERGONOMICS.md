# Desktop Ergonomics

Status: **plan**

This document is the canonical implementation plan for Vesper's high-frequency desktop ergonomics on top of the existing Caelestia + Hyprland desktop.

It consolidates selected interaction ideas from Omarchy and related desktop research into one Vesper-native plan. Omarchy is an interaction reference only. Vesper must reuse its existing shell, Settings, input, audio, notification, sharing, AI, compositor and Nix/systemd infrastructure instead of creating parallel subsystems.

Current implementation must always be inspected before claiming any target below is complete.

The physical Assistant/Copilot key and Quake Agent Console described below are target-only. The current Hyprland/QML tree has no `KEY_ASSISTANT` binding or `special:agent-console` workspace; the existing AI HUD shortcut is `Super + U -> codexbar-popup`.

## goals

The plan should make Vesper feel faster, safer and more coherent in everyday use without turning it into a pile of one-off scripts.

Primary goals:

- one semantic action may have several input triggers, but only one implementation;
- common actions should be reachable without opening a full Settings page;
- persistent configuration belongs in Settings; transient actions belong in the shell/command layer;
- existing Caelestia and Vesper backends remain authoritative;
- local state should be event-driven where the owner exposes reliable events;
- every visible state must remain truthful when a backend is unavailable or stale;
- keyboard-first workflows should not remove mouse/touchpad workflows;
- runtime conveniences must not silently mutate declarative Nix configuration;
- new UI must follow Vesper visual standards rather than copying Omarchy styling;
- application and agent failures should degrade locally instead of destabilizing the whole desktop session;
- shell reloads should not erase durable runtime truth merely because a view process restarted.

## explicit non-goals

The following are intentionally excluded from this plan:

- Night Light integration;
- sensitive-content filtering for clipboard history;
- Dictation;
- Tailscale integration as part of this ergonomics plan;
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
- the sibling `../../liquid-glass/docs/LIQUID-GLASS.md` contract when a consumer explicitly adopts it;
- reduced-motion, reduced-transparency and increased-contrast behavior where the owning surface supports it.

Rules:

- do not create an Omarchy-themed subsystem inside Vesper;
- do not invent a separate glass recipe for these features;
- avoid glass-on-glass nesting unless there is a semantic reason;
- prefer source-to-surface continuity and restrained movement over arbitrary fade-only transitions;
- transient surfaces must look related to the control that invoked them;
- indicators are views over authoritative state, not miniature independent state machines;
- QML renders normalized state and invokes bounded actions; it should not become the authoritative parser for system state;
- hardware-dependent rows must disappear or show truthful unsupported state rather than exposing fake controls.

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
  -> Text Size control
  -> per-monitor brightness capability presentation
  -> lid/clamshell policy presentation
  -> Input / Assistant-key discoverability
  -> Shortcuts / conflict detection

Vesper backends
  -> structured state normalization
  -> safe settings persistence
  -> CWD attribution
  -> app identity / launch-or-focus decisions where needed
  -> display preview/revert transaction
  -> DDC/CI capability and brightness routing
  -> clamshell state coordination
  -> app scope/process attribution
  -> bounded crash metadata collection

systemd user/session
  -> application scopes/slices where appropriate
  -> process/cgroup ownership
  -> user-level timers/inhibitors where already canonical
  -> OOM isolation only when real systemd-oomd policy can enforce it

PipeWire / WirePlumber
  -> audio device truth and default sink changes

LocalSend / OnionShare
  -> existing sharing transports

AI Hub / Agent Cockpit
  -> agent/provider/process status
  -> agent crash status/diagnosis entry point

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
agent.console.toggle
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
agent.console.toggle
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
- detailed quota/process views stay in AI / Agent Cockpit.

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

---

# 4. Stay Awake

Provide a temporary **Stay Awake** / caffeine action for inhibiting idle suspend/blanking.

Requirements:

- use the existing system inhibitor mechanism;
- do not rewrite global power policy;
- show clear active/inactive state;
- show a compact shell indicator while active;
- disabling immediately releases the inhibitor;
- normal power/idle policy remains authoritative.

Useful for transfers, builds, long-running agents, presentations and recording.

---

# 5. Audio Output Cycle

Add a fast action to cycle currently usable PipeWire output devices.

```text
Speakers -> Headphones -> HDMI -> Bluetooth -> Speakers
```

Requirements:

- derive candidates from PipeWire/WirePlumber truth;
- skip disconnected/unavailable sinks;
- reuse the same backend used by normal audio controls;
- show the selected output through the normal Vesper OSD/notification language;
- preserve the full audio picker for direct non-linear selection.

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

When a Quickshell/plugin panel is a visible card inside a monitor-sized transparent layer surface, prefer the trustworthy registered visible-card geometry instead of capturing the whole transparent layer.

---

# 9. Capture Hub

Keep screenshot, region capture, screen recording, OCR and color picking under one coherent capture namespace.

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
- stale/dead PID cannot produce a fabricated path.

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

---

# 12. Unified Text Scaling

Add one user-facing **Text Size** control that adjusts the readable UI scale consistently across Vesper-owned shell surfaces and supported desktop toolkits without conflating it with monitor pixel scale.

Target placement:

```text
Settings
└── Display
    └── Text Size
```

Conceptual UX:

```text
Text Size
[──────●──────] 100%

Applies to supported:
Shell · GTK · Qt · Terminal
```

Contract:

- expose one normalized semantic value, preferably percentage/steps rather than leaking toolkit-specific units;
- adapt that value to Caelestia/Quickshell typography, GTK text scaling, Qt text/application scaling where safely supported, and configured terminal font size;
- keep font family selection separate from text size;
- keep monitor scale/resolution separate from text size;
- preview changes live where the current Settings architecture supports safe live preview;
- persistent changes must use the structured Vesper configuration path;
- do not edit arbitrary GTK/Qt/terminal config text directly from QML;
- if one target cannot be updated safely, report partial support instead of pretending all targets changed.

Acceptance criteria:

- changing Text Size visibly affects the shell and supported apps after the expected live reload/restart boundary;
- 100% maps to Vesper's canonical defaults;
- resetting returns every managed target to the canonical baseline;
- display scale remains unchanged;
- unsupported apps/toolkits do not cause the setting to fail globally.

---

# 13. External Monitor Brightness via DDC/CI

External monitor brightness should behave like a first-class desktop brightness control when the monitor exposes a real DDC/CI path.

Target behavior for hardware brightness keys and shell controls:

```text
active/focused display
    ├─ internal panel -> kernel/native backlight path
    ├─ external + DDC/CI -> DDC/CI brightness
    └─ unsupported external -> truthful unsupported/no-op policy
```

Settings placement:

```text
Settings
└── Display
    └── <monitor>
        └── Brightness      # only when enforceable
```

Requirements:

- discover DDC/CI capability per physical external display;
- map monitor identity to the correct DDC/CI device robustly;
- never expose a writable slider merely because a monitor is external;
- brightness keys should operate on the focused/active monitor according to one documented policy;
- use the same Vesper brightness OSD for internal and external changes;
- cache capability/identity sensibly but invalidate on monitor hotplug/change;
- bound DDC/CI calls with timeouts so a broken monitor bus cannot freeze the shell;
- do not poll DDC/CI at high frequency merely to animate UI;
- if DDC/CI disappears mid-session, keep the last known state clearly stale/unsupported rather than hammering the bus.

Acceptance criteria:

- supported external monitor brightness changes from the normal brightness flow;
- internal laptop panel keeps using its canonical backlight backend;
- unplugging/replugging a monitor re-resolves capability;
- unsupported displays expose no fake slider;
- one hung DDC/CI device cannot stall Settings or the shell indefinitely.

---

# 14. Laptop Clamshell Mode

Vesper should treat laptop-with-external-monitor use as a first-class mode instead of relying on incidental lid behavior.

Policy ownership is split intentionally:

```text
Settings -> Power & Performance
  -> lid-close policy

Settings -> Display
  -> built-in display state
  -> external display arrangement/profile
```

Target lid-close policy:

```text
When lid closes
├── external display present -> keep session active, disable built-in panel when policy says so
└── no usable external display -> follow normal suspend/lock policy
```

Requirements:

- identify the built-in panel through real connector/hardware semantics, not only one hard-coded connector name;
- distinguish physically present usable external outputs from stale compositor entries;
- lid close/open must be idempotent;
- preserve/recover the intended built-in panel scale and layout when reopening the lid;
- avoid duplicate workspace migration on repeated lid events;
- keep mirror/extend semantics compatible with the same canonical Display model;
- do not leave the user with every display disabled;
- docking/undocking during clamshell mode must produce a safe, diagnosable fallback;
- normal suspend policy remains authoritative when no usable external display exists.

Acceptance criteria:

- closing the lid with a valid external display does not unnecessarily suspend when configured for clamshell use;
- reopening restores the built-in panel's intended scale/layout;
- removing the external display while closed cannot leave a hidden unusable session indefinitely;
- repeated lid events do not progressively corrupt monitor/workspace state.

---

# 15. Context-aware Universal Actions

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

---

# 16. Compositor Screen Zoom

Provide compositor-level zoom for inspecting small UI/accessibility use.

Requirements:

- zoom around cursor/focus using Hyprland-native capabilities;
- repeated action increments zoom in controlled steps;
- separate action resets immediately to 100%;
- active zoom may expose an OSD/indicator;
- reduced-motion mode uses minimal transition;
- never confuse zoom with `Settings -> Display` scale or Text Size.

---

# 17. Bar Scroll + OSD

Volume and brightness bar controls should normalize high-resolution wheel/touchpad input into predictable steps.

Requirements:

- accumulate small touchpad wheel deltas;
- apply a real step only when the configured threshold/notch is crossed;
- use the same logical increment as existing volume/brightness controls unless a shared setting says otherwise;
- trigger OSD only when a real step is applied;
- mouse wheel and touchpad should feel consistent;
- reuse the existing volume/brightness backend and OSD;
- external-monitor brightness must route through the same brightness action layer when DDC/CI is supported.

---

# 18. Launch-or-Focus

Dedicated app actions may focus an existing application instead of spawning duplicates when that matches app semantics.

Rules:

- desktop-entry/application identity is authoritative;
- distinguish single-instance/focus-first apps from apps where multiple windows are expected;
- webapp/TUI wrappers need stable app IDs, not title regexes;
- inspect Caelestia launcher/dock behavior first;
- reuse existing correct launch-or-focus behavior rather than duplicate it;
- do not create another application identity registry.

---

# 19. systemd Application Scopes and OOM Isolation

Vesper should isolate launched desktop applications into trustworthy process/cgroup ownership where the current session architecture permits it.

Goal:

```text
launcher / desktop entry
        ↓
canonical app identity
        ↓
user systemd app scope/slice
        ↓
application process tree
```

Why:

- a runaway browser/Electron/native app should be attributable as one application tree;
- App Inspector can obtain better process ownership and resource aggregation;
- a single application should not take the whole graphical session down merely because it shares an undifferentiated compositor cgroup;
- real systemd-oomd policy can act on isolated cgroups when configured correctly.

Requirements:

- inspect the existing UWSM/systemd launch topology before adding another scope layer;
- reuse an existing correct `app-*.scope`/slice model if the session already provides one;
- preserve desktop-entry identity in scope naming/metadata where possible without trusting arbitrary user-controlled titles;
- children spawned by the app should remain attributable to the same application tree unless they intentionally detach into another managed unit;
- App Inspector should consume the same canonical scope/process attribution rather than maintaining a second PID guesser;
- do not claim OOM isolation merely because a scope exists: systemd-oomd/ManagedOOM policy must actually be configured and tested;
- never assign aggressive memory limits globally without an explicit policy design;
- compositor, shell and critical session services must remain outside ordinary application kill domains;
- launch failures must fall back safely according to the existing launcher contract rather than silently dropping applications.

Acceptance criteria:

- supported launched apps have stable attributable cgroup ownership;
- App Inspector can aggregate processes/resources from the same source;
- killing/restarting one app scope does not kill the Vesper shell/session;
- OOM behavior is described as active only when the real policy is enabled and verified;
- multi-window apps remain one logical application where identity says they should.

---

# 20. Agent Crash Capture and Diagnosis

Unexpected coding-agent exits should become a first-class Agent Cockpit diagnostic event without turning Vesper into an invasive process recorder.

Target UX:

```text
Agent crashed
Codex · vesper · exit 1

[View diagnosis] [Restart]
```

Canonical owner:

```text
Agent Cockpit / AI
```

Capture only bounded, relevant local metadata such as:

- agent/runtime identity and version when known;
- exit status / signal;
- start time, crash time and elapsed duration;
- cwd/repository/branch when already attributable;
- PID/cgroup/scope identity;
- whether the kernel/systemd recorded an OOM or memory-pressure kill when attributable;
- a bounded tail of stderr/log output from the Vesper-owned launch path when available;
- last known Agent Cockpit state required to explain the failure.

Privacy and security rules:

- never dump the full process environment;
- never capture API keys, credential files, shell history or arbitrary home-directory contents;
- redact known secret-bearing fields before persistence/presentation;
- keep local crash evidence local by default;
- `Diagnose with AI` must show what context will leave the machine before any remote model call;
- do not upload raw logs automatically;
- bound retained crash history by count/size/time;
- crash capture must not keep an agent alive or interfere with its exit semantics.

Actions may include:

```text
Restart
Open Agent Cockpit
Open bounded logs
Diagnose with AI
Dismiss
```

Acceptance criteria:

- normal exits are distinguishable from unexpected crashes;
- exit code/signal is truthful;
- OOM attribution is shown only with real evidence;
- secrets/environment are absent from stored crash records;
- restarting uses the canonical agent launch path and does not bypass Default Agent/provider policy.

---

# 21. SSH Session Recovery and Terminal Hygiene

Vesper should make interactive SSH failure leave the local terminal in a clean state, especially after remote tmux/TUI/editor sessions.

This is a thin wrapper/integration around the user's normal SSH client, not a replacement SSH implementation.

Requirements:

- preserve the user's SSH config, host aliases, ProxyJump and authentication behavior;
- enable sensible client keepalive behavior only through documented/user-controlled configuration;
- after an abnormal disconnect, restore local terminal modes such as alternate screen, cursor visibility, mouse tracking and sane tty state when they were left dirty;
- cleanup must run on success, failure and signal interruption where possible;
- optional automatic reconnect may exist only for established interactive sessions, with bounded retry/backoff and a clear way to stop;
- never loop forever without visible state;
- do not auto-retry authentication failures or host-key failures as if they were transient network drops;
- tmux/herdr/editor reconnection logic remains application/session-specific and should not be guessed from window titles;
- ordinary `ssh` should remain available without Vesper recovery behavior when the user explicitly wants raw semantics.

Acceptance criteria:

- dropping a remote full-screen TUI does not leave mouse tracking/alternate-screen state stuck locally;
- host-key/authentication failures return immediately rather than reconnecting forever;
- transient network loss can offer or perform bounded reconnect according to policy;
- reconnect never changes the selected SSH host/config semantics.

---

# 22. Workspace Layout Snapshots

Allow saving/restoring useful workspace arrangements.

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

# 23. Unified Command Palette

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
Text Size
Brightness
Zoom
```

Rules:

- bare `Super` remains the primary entry point once unified; `Super + Space` remains an alternate Vicinae binding;
- application launch remains desktop-entry based;
- Settings results navigate to the real row rather than reimplement the setting inside the palette;
- privileged/destructive actions keep existing confirmation policy;
- aliases/keywords should live in a structured shared action registry;
- do not scatter duplicate search registries across QML files.

---

# 24. Keep-mapped Hidden Shell Surfaces

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

---

# 25. Shell Restart Continuity

A Caelestia/Quickshell reload should restart presentation, not silently erase durable runtime truth owned elsewhere.

Continuity candidates:

```text
notifications/history
Quick Reminders
Stay Awake inhibitor state
recording state
DND state when owned durably
Agent Console applications/workspace
agent process/crash state
active share/transfer state when the owning app exposes it
```

Contract:

- durable runtime state must be owned by the subsystem that can survive/reconstruct after shell restart;
- QML must not be the only copy of reminder, notification-history, recording or inhibitor truth when continuity is promised;
- on shell startup/reload, views rehydrate from authoritative state and mark unknown/stale states honestly;
- dismissed notifications must not return as new;
- running recordings/transfers/agents must not be killed merely because their indicator UI restarted;
- a shell crash during a Display confirmation transaction must not convert a temporary preview into persistent config;
- continuity must not resurrect runtime actions that the owner already ended;
- do not persist every transient animation/popup just for visual continuity.

Acceptance criteria:

- restarting the shell preserves/reconstructs the states explicitly covered by this contract;
- no duplicate reminders/notifications are emitted solely because the shell restarted;
- indicators return to the real state after restart;
- critical owners remain independent of view lifecycle.

---

# 26. Shell State Indicators

Show compact indicators only for temporary user-controlled states that are easy to forget.

Candidate states:

```text
Stay Awake
Recording
DND
active reminders
screen zoom
clamshell state when useful
Tor/privacy state when already provided by Privacy HUD
active-agent count / quota pressure when already provided by AI
agent crash attention state when one exists
```

Rules:

- indicator reads authoritative state;
- inactive low-priority indicators may disappear;
- clicking opens the owning control/status surface where useful;
- do not create new collectors just to draw an icon;
- Night Light and Dictation remain excluded.

---

# 27. Event-driven Local State

Reduce avoidable short-interval subprocess polling for local desktop state.

Prefer events, DBus/signals, sockets, compositor callbacks or long-lived subscriptions for:

- audio default-sink/device changes;
- network connectivity;
- monitor layout/focus and lid state;
- power/session idle state;
- notification state;
- recording state;
- app scope/process lifecycle;
- live agent process/crash state when Agent Cockpit exposes it.

Bounded polling remains acceptable for inherently remote/periodic state such as provider quota refreshes or hardware interfaces that expose no usable event mechanism.

The rule is not "no polling". The rule is: do not repeatedly spawn expensive local probes when the owner can push a trustworthy change.

---

# Settings integration

Do not create an "Omarchy features" page. Each persistent/discoverable control belongs to its real Settings owner.

Target information placement:

```text
Settings
├── Apps
│   ├── Default Apps
│   │   └── Default Agent
│   └── App Inspector
│       └── process/scope/resource ownership when implemented
│
├── Display
│   ├── Arrange Displays
│   ├── Text Size
│   ├── per-monitor Brightness when supported
│   ├── built-in display state / clamshell status
│   ├── resolution / refresh / scale / orientation
│   ├── mirror / extend
│   ├── workspace assignment
│   └── saved monitor profiles
│
├── Power & Performance
│   ├── normal idle/suspend policy
│   └── lid-close / clamshell policy
│
├── Input
│   └── Assistant / Copilot key
│       ├── detected hardware state
│       └── action: Toggle Agent Console
│
├── Shortcuts
│   ├── Toggle Agent Console
│   ├── Move window to Agent Console
│   ├── Share
│   ├── Reminder
│   ├── Audio Output Cycle
│   ├── Zoom / Reset Zoom
│   ├── notification actions
│   └── universal semantic actions when implemented
│
└── AI
    └── Agents / Agent Cockpit
        └── crash history / diagnosis entry point
```

Settings rules:

- Default Agent is the only persistent agent selector for generic agent launch;
- physical Assistant/Copilot-key UI appears only when actually detected;
- its default semantic action is `Toggle Agent Console`;
- changing key behavior later must use the canonical shortcut registry/conflict checker;
- Display Arrange is part of Display, not a separate application;
- Text Size is independent from monitor scale and font-family selection;
- DDC/CI brightness rows exist only for monitors with verified writable capability;
- lid-close policy lives under Power & Performance while monitor state remains visible in Display;
- display confirmation/revert is a Settings transaction, not a hidden fire-and-forget script;
- CWD-aware launch, bar-scroll normalization, Launch-or-Focus and shell continuity are interaction/runtime behavior, not toggles by default;
- compositor zoom belongs in Shortcuts, not persistent Display scale;
- Stay Awake is temporary runtime state, not a replacement for power policy;
- app scopes are runtime architecture; expose observability in App Inspector instead of a fake per-app isolation switch before enforcement exists;
- agent crash capture is local by default and remote AI diagnosis is explicit;
- LocalSend/OnionShare retain their existing app/service configuration ownership.

`APPS-SETTINGS.md` remains authoritative for the Default Apps/App Inspector surface. `SETTINGS.md` remains authoritative for the wider Settings information architecture. `AI.md` remains authoritative for provider/model/capability policy.

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

The shortcut registry should expose semantic action IDs so hardware keys, user chords, command-palette actions and Settings all point at the same action.

Example conceptual IDs:

```text
agent.console.toggle
agent.console.move-current-window
agent.crash.open-latest
share.open
reminder.create
stay-awake.toggle
audio.output.next
capture.open
capture.ocr
capture.color-picker
display.zoom.in
display.zoom.reset
display.brightness.adjust
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
2. inventory current app identity/launch/UWSM/systemd scope behavior;
3. inventory capture backend and popup geometry ownership;
4. inventory notification history/actions/DND support;
5. inventory PipeWire audio APIs already exposed to QML/backend;
6. inventory Settings row/navigation primitives;
7. inventory display/backlight/DDC/CI/lid event capabilities;
8. inventory shell-owned vs externally-owned runtime state for restart continuity;
9. identify the canonical structured shortcut/action registry or create one if missing;
10. identify which local-state pollers can be replaced with existing events;
11. document conflicts instead of silently replacing working behavior.

Exit criteria:

- each feature has an owner and no duplicate backend is planned;
- final shortcut candidates are conflict-checked;
- Settings navigation targets are known;
- existing functionality that already satisfies a target is marked reuse, not rebuilt;
- app scope/OOM design is based on the real current session topology;
- hardware-dependent controls have explicit unsupported behavior.

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

## Phase 2 — everyday utility actions

Implement:

1. Share menu with LocalSend + OnionShare;
2. Stay Awake;
3. Audio Output Cycle;
4. notification actions/replay;
5. Quick Reminders;
6. matching temporary shell indicators.

## Phase 3 — capture and launch ergonomics

Implement:

1. Keyboard-first Smart Capture;
2. shell/plugin visible-card geometry targets;
3. Color Picker integration;
4. Capture result Share actions;
5. CWD-aware terminal launch;
6. CWD-aware file-manager launch;
7. Launch-or-Focus audit and first safe app actions;
8. SSH terminal-cleanup/recovery wrapper.

## Phase 4 — display and laptop polish

Implement:

1. Display Arrange native editor;
2. safe temporary apply + exact revert;
3. Unified Text Scaling;
4. DDC/CI external-monitor brightness routing;
5. clamshell/lid policy and safe monitor recovery;
6. compositor screen zoom;
7. bar scroll delta normalization and OSD consistency;
8. persistence handoff only through structured runtime-to-declarative paths.

## Phase 5 — process and agent resilience

Implement:

1. verify/reuse or introduce canonical per-app systemd scopes;
2. connect scope/process attribution to App Inspector;
3. define/test real systemd-oomd policy before claiming OOM isolation;
4. add bounded Agent Crash Capture;
5. expose local crash history/status in Agent Cockpit;
6. add explicit `Diagnose with AI` handoff with context review/redaction.

## Phase 6 — command and workflow convergence

Implement:

1. unified structured action registry across shell/Settings;
2. unified command palette;
3. semantic Settings navigation results;
4. workspace layout snapshots;
5. context-aware universal actions after binding/app identity audit.

## Phase 7 — continuity, performance and event cleanup

After user-facing behavior is stable:

1. move promised durable runtime truth out of view-only QML state;
2. verify shell-restart rehydration and no duplicate events;
3. profile Quickshell show/hide/map/unmap costs;
4. apply keep-mapped parking only where measurement supports it;
5. replace avoidable local subprocess polling with owner events/signals;
6. verify multi-monitor/capture/clamshell behavior after optimization;
7. remove superseded helper scripts/dead code only after proving no callers remain.

---

# testing and acceptance matrix

Every implementation PR should test the behavior it introduces rather than only syntax/build success.

## shell/input

Test:

- shortcut conflicts and action-ID routing;
- physical Assistant key absent/present states;
- no accidental F23 hijacking;
- special-workspace toggle idempotency;
- multi-window console tiling;
- console persistence across hide/show.

## scaling and monitors

Test:

- 1.0, common fractional and 2.0 scale;
- reserved top bar area;
- focus moving between differently scaled monitors;
- monitor connect/disconnect;
- Display Arrange auto-revert;
- Text Size without changing monitor scale;
- DDC/CI supported/unsupported/hung devices;
- internal-panel backlight path;
- lid close/open with and without external displays;
- docking/undocking while clamshell policy is active.

## capture

Test:

- mouse selection;
- Tab/arrow navigation;
- Enter/Ctrl+Enter;
- empty secondary monitor;
- shell/plugin visible-card capture;
- cancellation cleanup;
- screenshot/record target parity where supported.

## audio/brightness/OSD

Test:

- mouse wheel and high-resolution touchpad scroll;
- disconnected sink skipped;
- actual selected sink/value reflected in OSD;
- external brightness and internal brightness use one semantic feedback path;
- no duplicate backend state.

## reminders/notifications/continuity

Test:

- shell restart;
- reminder persistence/cancel;
- DND history behavior;
- dismiss latest/all;
- actionable notification validity;
- bounded history;
- replay does not manufacture new app events;
- shell restart does not duplicate reminders/notifications;
- recording/Stay Awake/agent state rehydrates truthfully when continuity is promised.

## launch/process resilience

Test:

- supported terminal CWD;
- unknown terminal and dead/stale PID fallback;
- file manager opens same resolved directory;
- Launch-or-Focus does not collapse apps that need multiple windows;
- app scope ownership is stable;
- killing one app scope leaves shell/session alive;
- OOM claim is disabled unless real policy is configured;
- SSH disconnect cleans local terminal state;
- auth/host-key SSH failures do not auto-reconnect forever.

## agent crash diagnosis

Test:

- clean/normal exit vs crash distinction;
- exit code and signal attribution;
- bounded log tail;
- real OOM evidence path;
- no environment/API-key leakage;
- crash-history retention bound;
- AI diagnosis requires explicit context review/handoff.

## UI/accessibility

Test new surfaces with bright/dark/high-frequency backgrounds, reduced motion, reduced transparency where applicable, increased contrast, and keyboard-only operation for flows that claim it.

---

# failure and fallback rules

Use truthful degradation throughout the plan.

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

DDC/CI unsupported or timed out
  -> no fake brightness write; surface becomes unsupported/stale

External display disappears during clamshell
  -> safe display/power fallback; never intentionally leave user with no usable output

App scope unavailable
  -> use verified existing launch path; do not claim isolation

OOM evidence unavailable
  -> crash cause remains unknown

SSH authentication/host-key failure
  -> stop; do not treat as transient reconnect

OnionShare not installed
  -> unavailable/hidden according to normal action policy

notification action expired
  -> do not invoke

plugin capture geometry unavailable
  -> use the safe existing capture target; never invent coordinates

shell state owner unavailable after reload
  -> show unknown/stale, do not recreate a guessed runtime action
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
- current external brightness value unless the hardware/backend already owns persistence;
- notification DND runtime state according to its existing owner;
- temporary display preview before confirmation;
- active clamshell runtime state derived from lid/output state.

Persistent examples when supported:

- Default Agent;
- shortcut customization;
- Text Size;
- lid-close/clamshell policy;
- accepted monitor layout/profile through the structured Vesper configuration path;
- saved workspace layout snapshots.

Durable local operational records may include bounded notification/reminder state and bounded agent crash records, but they are not declarative Nix configuration.

No QML surface should silently write arbitrary Nix/Lua/toolkit config text. Persistent system changes use the guarded Vesper runtime-to-declarative contract.

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
13 SSH session cleanup/recovery
14 Display Arrange + automatic revert
15 Unified Text Scaling
16 External-monitor DDC/CI brightness
17 Clamshell/lid handling
18 Compositor screen zoom
19 Bar scroll normalization + OSD
20 Launch-or-Focus
21 systemd application scopes / App Inspector attribution
22 verified OOM isolation policy
23 Agent Crash Capture + Diagnosis
24 Unified command palette/action registry convergence
25 Workspace layout snapshots
26 Context-aware universal actions
27 Shell restart continuity
28 Shell state indicator polish
29 Event-driven cleanup
30 Keep-mapped shell-surface optimizations where benchmarked
```

The ordering is dependency-aware: establish canonical state/actions first, ship low-risk utility wins next, then display/laptop resilience, process/agent isolation and workflow convergence, and optimize only after behavior is correct.
