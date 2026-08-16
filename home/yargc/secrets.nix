{ config, ... }:
{
  # The private age key is machine-local and never belongs in the repository.
  # Add encrypted files and individual sops.secrets entries only when a secret
  # is actually needed. See docs/SECRETS.md for the bootstrap flow.
  sops.age.keyFile = "${config.home.homeDirectory}/.config/sops/age/keys.txt";
}
