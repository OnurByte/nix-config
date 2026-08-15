{
  pkgs,
  username,
  ...
}:
{
  # XAMPP-equivalent, but native to NixOS: Apache + PHP + MariaDB.
  # The web server is deliberately loopback-only for local development.
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

  systemd.tmpfiles.rules = [
    "d /srv/http 0755 ${username} users - -"
  ];
}
