{
  lib,
  pkgs,
  username,
  ...
}:
{
  # XAMPP-equivalent, but native to NixOS: Apache + PHP + MariaDB.
  # The services are installed and configured declaratively, but Vesper keeps
  # them stopped at boot. `web-start` activates the target on demand.
  services.httpd = {
    enable = true;
    enablePHP = true;
    mpm = "prefork";

    virtualHosts.localhost = {
      documentRoot = "/srv/http";
      listen = [
        {
          ip = "127.0.0.1";
          port = 80;
          ssl = false;
        }
      ];
    };
  };

  services.mysql = {
    enable = true;
    package = pkgs.mariadb;
    ensureDatabases = [ "dev" ];
  };

  # The NixOS modules normally hook these services into multi-user.target.
  # Remove that boot-time activation and group them behind one explicit target.
  systemd.services.httpd = {
    wantedBy = lib.mkForce [ ];
    partOf = [ "vesper-web.target" ];
  };

  systemd.services.mysql = {
    wantedBy = lib.mkForce [ ];
    partOf = [ "vesper-web.target" ];
  };

  systemd.targets.vesper-web = {
    description = "Vesper local Apache + MariaDB development stack";
    wants = [
      "mysql.service"
      "httpd.service"
    ];
    after = [ "mysql.service" ];
  };

  systemd.tmpfiles.rules = [
    "d /srv/http 0755 ${username} users - -"
  ];
}
