{ username, ... }:
let
  zapretProfile = {
    name = "default";
    ownership = "nix";
    mutable = false;
    hostAutodetect = true;
    maxPackets = 16;
    tcpPorts = [ 443 ];
    udpPorts = [ ];
    payload = "tls_client_hello";
    parameters = [
      "--filter-tcp=443"
      "--payload=tls_client_hello"
      "--lua-desync=fake:blob=fake_default_tls:tcp_ts=-1000:repeats=1"
      "--lua-desync=fakedsplit:pos=1,midsld:tcp_ts=-1000"
    ];
  };
in
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
      inherit (zapretProfile) maxPackets tcpPorts udpPorts;
    };

    profiles.default = {
      hosts.autodetect.enable = zapretProfile.hostAutodetect;
      inherit (zapretProfile) parameters;
    };
  };

  # Export the exact declarative profile to the Settings control plane instead
  # of duplicating Zapret knobs in QML or a mutable runtime config.
  environment.etc."vesper/zapret-profile.json".text = builtins.toJSON zapretProfile;

  # Settings may only start/stop the one Zapret worker it exposes. Do not grant
  # generic systemd service-management rights to the desktop session.
  security.polkit = {
    enable = true;
    extraConfig = ''
      polkit.addRule(function(action, subject) {
        if (action.id == "org.freedesktop.systemd1.manage-units" &&
            subject.user == "${username}" &&
            action.lookup("unit") == "nfqws2@default.service") {
          var verb = action.lookup("verb");
          if (verb == "start" || verb == "stop") {
            return "yes";
          }
        }
      });
    '';
  };
}
