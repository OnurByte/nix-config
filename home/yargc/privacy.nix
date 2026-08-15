{ pkgs, ... }:
{
  home.packages = with pkgs; [
    # Monero reference stack. monero-cli also provides monerod,
    # monero-wallet-cli and monero-wallet-rpc without enabling a background node.
    monero-cli
    monero-gui

    # Lightweight desktop alternative with integrated Tor support.
    feather
  ];
}
