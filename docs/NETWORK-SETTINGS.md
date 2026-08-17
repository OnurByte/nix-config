# network settings

Caelestia remains the only network UI.

Vesper extends the native Nexus Network page with:

- airplane mode that switches NetworkManager radios and Bluetooth together
- QR export for the currently active Wi-Fi profile
- the existing Caelestia ethernet and VPN controls
- a process proxy written to `~/.config/environment.d/90-vesper-proxy.conf`
- Zapret2 service/profile status

The proxy setting affects newly started processes. A session restart is the clean handoff when every desktop process should inherit it.

Zapret2 stays declarative in Nix. The UI reports the current narrow adaptive profile instead of inventing mutable presets that would diverge from `modules/privacy/default.nix`.
