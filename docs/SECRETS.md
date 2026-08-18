# secrets

Status: **current**

Vesper uses more than one secret mechanism because different consumers have different ownership and lifetime requirements.

Use this split consistently:

```text
interactive/shared AI provider API keys
  -> freedesktop Secret Service
  -> managed through vesper-control

declarative user services and MCP secrets
  -> sops-nix
  -> machine-local age private key

Restic system backup credentials
  -> root-owned machine-local files
  -> /etc/vesper/restic.env + Restic password file
```

Do not duplicate one secret across mechanisms unless a real consumer requires a different delivery method.

## AI provider credentials

OpenAI, Anthropic, xAI, OpenRouter and Google AI shared provider keys belong to the Vesper AI credential manager described in `AI.md`.

They are stored through freedesktop Secret Service with `secret-tool` and consumed through `vesper-control`.

Do not configure those shared AI keys through `sops-nix` merely because sops-nix is available. A sops secret does not automatically configure the Vesper AI control plane.

## sops-nix

Vesper uses `sops-nix` for declarative secrets that need file-backed delivery to Home Manager-managed consumers such as MCP servers or user services.

The private age key never goes in Git. Encrypted `*.sops.yaml` files can be committed.

### first setup

Create the local age key once:

```bash
mkdir -p ~/.config/sops/age
age-keygen -o ~/.config/sops/age/keys.txt
chmod 600 ~/.config/sops/age/keys.txt
```

Print the public recipient:

```bash
age-keygen -y ~/.config/sops/age/keys.txt
```

It prints a value beginning with `age1...`.

Create `.sops.yaml` at the repository root and replace the example recipient with that public key:

```yaml
keys:
  - &vesper age1REPLACE_WITH_YOUR_PUBLIC_KEY

creation_rules:
  - path_regex: secrets/.*\.sops\.yaml$
    key_groups:
      - age:
          - *vesper
```

Only the public age recipient belongs in `.sops.yaml`.

### create a secret file

```bash
mkdir -p secrets
sops secrets/vesper.sops.yaml
```

For example, a declarative MCP/service secret may look like:

```yaml
context7_api_key: example-secret
```

Saving from `sops` writes encrypted values. Check the file before committing it. Plain values should not be visible.

### expose a secret to Home Manager

`home/yargc/secrets.nix` already points sops-nix at the machine-local age key.
Once `secrets/vesper.sops.yaml` exists, extend that file for the actual declarative consumer:

```nix
{ config, ... }:
{
  sops = {
    age.keyFile = "${config.home.homeDirectory}/.config/sops/age/keys.txt";
    defaultSopsFile = ../../secrets/vesper.sops.yaml;

    secrets.context7_api_key = { };
  };
}
```

Then switch:

```bash
nh os switch
```

The Home Manager module decrypts user secrets into the user runtime directory and exposes stable secret paths through `config.sops.secrets.<name>.path`.

Do not put the decrypted value directly in `home.sessionVariables` or Nix source. Point the consuming process at the secret file instead.

### use a secret from an MCP server

Home Manager's MCP registry understands file-backed environment variables.
A server can consume a sops-nix secret without writing the value into generated config:

```nix
programs.mcp.servers.context7 = {
  # existing command/args omitted
  env.CONTEXT7_API_KEY.file = config.sops.secrets.context7_api_key.path;
};
```

Codex, Claude Code and OpenCode receive that through their existing MCP integration.

## Restic secrets

Restic is a system backup service and deliberately keeps its credentials outside the Nix store in root-owned machine-local files.

The operational source of truth is `BACKUP.md`.

Do not move Restic credentials into Home Manager sops secrets unless the backup service ownership is intentionally redesigned at the same time.

## edit and rotate

Edit an encrypted sops file:

```bash
sops secrets/vesper.sops.yaml
```

If the age recipient changes, update `.sops.yaml` and rotate the file recipients:

```bash
sops updatekeys secrets/vesper.sops.yaml
```

Back up `~/.config/sops/age/keys.txt` somewhere encrypted. Losing that private key without another configured recipient means losing access to the encrypted secrets.

## guardrails

Never put decrypted secrets in:

- Git
- Nix source literals
- shell history
- command-line arguments
- broad session environment variables
- documentation examples that resemble real credentials

Use the narrowest delivery mechanism owned by the actual consumer.
