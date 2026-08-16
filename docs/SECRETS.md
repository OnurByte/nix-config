# secrets

Vesper uses `sops-nix` with a machine-local age key.

The private age key never goes in Git. Encrypted `*.sops.yaml` files can be committed.

## first setup

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

## create a secret file

```bash
mkdir -p secrets
sops secrets/vesper.sops.yaml
```

For example, edit it to contain:

```yaml
openai_api_key: sk-example
anthropic_api_key: sk-ant-example
```

Saving from `sops` writes encrypted values. Check the file before committing it. Plain values should not be visible.

## expose a secret to Home Manager

`home/yargc/secrets.nix` already points sops-nix at the machine-local age key. Once `secrets/vesper.sops.yaml` exists, extend that file like this:

```nix
{ config, ... }:
{
  sops = {
    age.keyFile = "${config.home.homeDirectory}/.config/sops/age/keys.txt";
    defaultSopsFile = ../../secrets/vesper.sops.yaml;

    secrets.openai_api_key = { };
    secrets.anthropic_api_key = { };
  };
}
```

Then switch:

```bash
nh os switch
```

The Home Manager module decrypts user secrets into the user runtime directory and exposes stable secret paths through `config.sops.secrets.<name>.path`.

Do not put the decrypted value directly in `home.sessionVariables` or Nix source. Point the consuming process at the secret file instead.

## use a secret from an MCP server

Home Manager's MCP registry understands file-backed environment variables. A server can consume a sops-nix secret without writing the value into generated config:

```nix
programs.mcp.servers.example = {
  command = "/path/to/server";
  env.API_KEY.file = config.sops.secrets.openai_api_key.path;
};
```

Codex, Claude Code and OpenCode receive that through their existing MCP integration.

## edit and rotate

Edit the encrypted file:

```bash
sops secrets/vesper.sops.yaml
```

If the age recipient changes, update `.sops.yaml` and rotate the file recipients:

```bash
sops updatekeys secrets/vesper.sops.yaml
```

Back up `~/.config/sops/age/keys.txt` somewhere encrypted. Losing that private key without another configured recipient means losing access to the encrypted secrets.
