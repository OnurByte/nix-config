{ ... }:
{
  # Keep a system Tor SOCKS endpoint available for CLI/privacy-aware software.
  # Tor Browser remains a separate application with its own bundled Tor.
  services.tor = {
    enable = true;
    client.enable = true;
  };

  # Narrow, adaptive anti-DPI handling. Only TLS ClientHello traffic on TCP/443
  # enters NFQUEUE; the host autodetector persists destinations that actually
  # appear to need the bypass instead of mangling every connection forever.
  services.zapret2 = {
    enable = true;

    firewall = {
      # This is a single-host policy. Keep VPN, container, loopback and Wi-Fi
      # P2P interfaces out of the NFQUEUE path.
      interfaces = [
        "wlan0"
        "enp2s0"
      ];
      maxPackets = 16;
      tcpPorts = [ 443 ];
      udpPorts = [ ];
    };

    profiles.default = {
      hosts.autodetect.enable = true;
      parameters = [
        "--filter-tcp=443"
        "--payload=tls_client_hello"
        "--lua-desync=fake:blob=fake_default_tls:tcp_ts=-1000:repeats=1"
        "--lua-desync=fakedsplit:pos=1,midsld:tcp_ts=-1000"
      ];
    };
  };
}
