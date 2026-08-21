# network settings

Status: **current**

Caelestia remains the only network UI.

Vesper extends the native Nexus Network page with:

- airplane mode that switches NetworkManager radios and Bluetooth together
- QR export for the currently active Wi-Fi profile
- the existing Caelestia ethernet and VPN controls
- a process proxy written to `~/.config/environment.d/90-vesper-proxy.conf`
- Zapret2 service/profile status

## proxy environment

The proxy backend writes the user-session environment contract to:

```text
~/.config/environment.d/90-vesper-proxy.conf
```

Do not describe this as an immediate mutation of the already-running desktop environment. A clean user-session restart is the reliable global handoff for compositor-launched and other desktop processes to inherit the new values.

Proxy status is derived from the effective `environment.d` file itself. The backend writes that file atomically with private `0600` permissions and no longer commits a separate configured-state marker before the effective write succeeds. `proxy_clear` still removes the legacy marker left by older builds.

Remaining operational rules:

- validate the requested proxy URL before mutating state
- write the effective environment file through a private temporary file and atomic rename
- keep any proxy file that may contain credentials private to the user
- derive configured state from the effective file
- clearing the proxy must remove effective and bookkeeping state coherently
- do not claim that the current desktop session has inherited a new proxy until that is actually true

## airplane mode

Airplane mode now captures Wi-Fi, WWAN and Bluetooth state under the user runtime directory, disables each available radio explicitly and restores the captured state on exit. The displayed network state includes WWAN and records whether a Bluetooth controller is available. Live hardware round-trip proof is still pending.

Remaining operational rules:

- keep the captured state runtime-only
- disable the intended radios and report failures instead of silently ignoring them
- on exit restore the captured state rather than blindly turning every radio on
- include WWAN in airplane truth when the hardware/backend exposes it
- do not overwrite a radio the user intentionally kept disabled before airplane mode

Any saved pre-airplane state is runtime state only. It must not become a second declarative network configuration.

## Wi-Fi QR

QR export is for the currently active Wi-Fi profile.

The production package currently carries a patch that feeds the QR payload to `qrencode` through stdin rather than placing the Wi-Fi credential payload in process arguments. That security correction belongs in the tracked canonical `vesper-control` Rust source, not only in packaging-time patch assembly.

When the source is consolidated, keep the stdin behavior and remove the redundant patch path so reviewed source and shipped runtime stay identical.

## Zapret2

Zapret2 stays declarative in Nix. The UI reports the current narrow adaptive profile instead of inventing mutable presets that would diverge from `modules/privacy/default.nix`.
