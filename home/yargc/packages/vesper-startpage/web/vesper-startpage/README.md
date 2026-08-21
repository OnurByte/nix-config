# vesper startpage

This is the browser surface served by `vesper-startpage` on
`http://127.0.0.1:3210/`.

The Home Manager unit keeps only the loopback socket active. The Rust backend
is socket-activated on the first JavaScript API request.

It renders the local Helium shortcuts, filtered Zen and Helium history,
read-only Hermes research output and Tor links. Onion links are opened by the
Rust service through the configured Tor Browser executable; they are not
normal browser anchors. The Nix-managed Tor Browser launcher also opens this
same page when started without a URL.

Run the frontend checks from this directory:

```bash
bun run selfcheck
bun run typecheck
bun run lint
bun run build
```

The Nix package installs the checked-in `dist/` output and compiles the
loopback Rust server from the sibling package directory.
