{ inputs, pkgs, ... }:
{
  home.packages = with pkgs; [
    # Monero reference stack. monero-cli also provides monerod,
    # monero-wallet-cli and monero-wallet-rpc without enabling a background node.
    monero-cli
    monero-gui

    # Lightweight desktop alternative with integrated Tor support.
    feather

    # BTC <-> XMR atomic-swap desktop wallet from nixpkgs.
    eigenwallet

    # Rust alternative Monero node implementation. Installed for opt-in use;
    # never started as a background service by this Home Manager profile.
    inputs.self.packages.${pkgs.system}.cuprated
  ];
}
