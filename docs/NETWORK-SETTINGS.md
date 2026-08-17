# network settings

Caelestia remains the only network UI.

Vesper extends the native Nexus Network page with:

- airplane mode that switches Wi-Fi, WWAN and Bluetooth together, then restores their previous radio states when disabled
- QR export for the currently active Wi-Fi profile, with the QR payload sent to `qrencode` over stdin rather than process arguments
- the existing Caelestia Ethernet and VPN controls
- a process proxy written to `~/.config/environment.d/90-vesper-proxy.conf`
- a native Zapret2 tuning page

The proxy setting affects newly started processes. A session restart is the clean handoff when every desktop process should inherit it.

## Zapret2 tuning

Nix still owns Zapret2, nftables and the interception boundary:

- TCP 443 only
- first 16 packets only
- no UDP interception
- adaptive host detection remains enabled

The UI can change only two runtime nfqws2 strategy parameters:

- fake replay count: `1`, `2`, `4` or `6`
- split pattern: `1,midsld`, `method+2,midsld` or `1,sniext+1,midsld`

These form 12 allowlisted combinations. The desktop user cannot pass arbitrary root-side nfqws2 arguments or widen the nftables rules. A small root-owned state token under `/var/lib/vesper-zapret/profile` selects the strategy, and `nfqws2@default.service` is restarted after a valid change. Reset removes that runtime token and returns to the Nix default (`1` repeat and `1,midsld`).
