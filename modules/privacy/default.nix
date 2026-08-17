{ config, lib, pkgs, username, ... }:
let
  zapret = config.services.zapret2;

  zapretRunner = pkgs.writeShellScript "vesper-zapret-run" ''
    set -eu
    token=r1-default
    if [ -r /var/lib/vesper-zapret/profile ]; then
      IFS= read -r token < /var/lib/vesper-zapret/profile || true
    fi
    case "$token" in
      r1-default) repeats=1; split='1,midsld' ;;
      r2-default) repeats=2; split='1,midsld' ;;
      r4-default) repeats=4; split='1,midsld' ;;
      r6-default) repeats=6; split='1,midsld' ;;
      r1-method) repeats=1; split='method+2,midsld' ;;
      r2-method) repeats=2; split='method+2,midsld' ;;
      r4-method) repeats=4; split='method+2,midsld' ;;
      r6-method) repeats=6; split='method+2,midsld' ;;
      r1-sni) repeats=1; split='1,sniext+1,midsld' ;;
      r2-sni) repeats=2; split='1,sniext+1,midsld' ;;
      r4-sni) repeats=4; split='1,sniext+1,midsld' ;;
      r6-sni) repeats=6; split='1,sniext+1,midsld' ;;
      *) repeats=1; split='1,midsld' ;;
    esac
    exec ${lib.getExe zapret.package} \
      --qnum=${toString zapret.firewall.queue} \
      --fwmark=${zapret.firewall.desyncFwmark} \
      --lua-init=@${zapret.package}/share/zapret2/lua/zapret-lib.lua \
      --lua-init=@${zapret.package}/share/zapret2/lua/zapret-antidpi.lua \
      --name=default \
      --filter-tcp=443 \
      --payload=tls_client_hello \
      "--lua-desync=fake:blob=fake_default_tls:tcp_ts=-1000:repeats=$repeats" \
      "--lua-desync=fakedsplit:pos=$split:tcp_ts=-1000" \
      --hostlist-auto=/var/lib/zapret2/default-hosts.txt
  '';

  zapretSetter = pkgs.writeShellScript "vesper-zapret-set" ''
    set -eu
    token="$1"
    case "$token" in
      reset)
        ${pkgs.coreutils}/bin/rm -f /var/lib/vesper-zapret/profile
        ;;
      r1-default|r2-default|r4-default|r6-default|r1-method|r2-method|r4-method|r6-method|r1-sni|r2-sni|r4-sni|r6-sni)
        tmp=/var/lib/vesper-zapret/profile.tmp
        printf '%s\n' "$token" > "$tmp"
        ${pkgs.coreutils}/bin/chmod 0644 "$tmp"
        ${pkgs.coreutils}/bin/mv -f "$tmp" /var/lib/vesper-zapret/profile
        ;;
      *) echo "unsupported Zapret2 tuning token" >&2; exit 2 ;;
    esac
    ${pkgs.systemd}/bin/systemctl restart nfqws2@default.service
  '';
in
{
  services.tor = {
    enable = true;
    client.enable = true;
  };

  services.zapret2 = {
    enable = true;
    firewall = {
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

  systemd.services."nfqws2@default".serviceConfig.ExecStart = lib.mkForce [
    ""
    "${zapretRunner}"
  ];

  systemd.tmpfiles.rules = [ "d /var/lib/vesper-zapret 0755 root root -" ];

  systemd.services."vesper-zapret-profile@" = {
    description = "Apply Vesper Zapret2 tuning profile %i";
    serviceConfig = {
      Type = "oneshot";
      ExecStart = "${zapretSetter} %i";
      NoNewPrivileges = true;
      PrivateTmp = true;
      ProtectHome = true;
      ProtectSystem = "strict";
      ReadWritePaths = [ "/var/lib/vesper-zapret" ];
      RestrictAddressFamilies = [ "AF_UNIX" ];
      UMask = "0022";
    };
  };

  security.polkit = {
    enable = true;
    extraConfig = ''
      polkit.addRule(function(action, subject) {
        if (action.id !== "org.freedesktop.systemd1.manage-units" ||
            subject.user !== "${username}" ||
            action.lookup("verb") !== "start") {
          return polkit.Result.NOT_HANDLED;
        }
        var unit = action.lookup("unit");
        if (unit && unit.indexOf("vesper-zapret-profile@") === 0 && unit.slice(-8) === ".service") {
          return polkit.Result.YES;
        }
        return polkit.Result.NOT_HANDLED;
      });
    '';
  };
}
