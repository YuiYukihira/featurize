{ inputs, cell }: {
  mailslurper = { config, lib, pkgs, ... }:
    with lib;
    let
      cfg = config.services.mailslurper;
      configFileData = {
        wwwAddress = "127.0.0.1";
        wwwPort = cfg.config.wwwPort;
        serviceAddress = "127.0.0.1";
        servicePort = cfg.config.servicePort;
        smtpAddress = "127.0.0.1";
        smtpPort = cfg.config.smtpPort;
        dbEngine = "SQLite";
        dbHost = "";
        dbPort = 0;
        dbDatabase = ".data/mailslurper.db";
        dbUserName = "";
        dbPassword = "";
        maxWorkers = 1000;
      };
      configFile =
        pkgs.writeText "config.json" (lib.generators.toJSON { } configFileData);
      start-command = pkgs.writeShellScriptBin "start-mailslurper" ''
        ${cfg.package}/bin/mailslurper -config ${configFile}
      '';
    in {
      options.services.mailslurper = {
        enable = mkEnableOption "Enable the service";
        package = mkOption {
          type = types.package;
          default = cell.packages.mailslurper;
          description = "Package to use";
        };
        config = {
          wwwPort = mkOption {
            type = types.int;
            default = 8080;
          };
          servicePort = mkOption {
            type = types.int;
            default = 8888;
          };
          smtpPort = mkOption {
            type = types.int;
            default = 2525;
          };
        };
      };

      config = {
        __services.mailslurper = {
          command = "${start-command}/bin/start-mailslurper";
          enable = cfg.enable;
        };

        commands = [{ package = cell.packages.mailslurper; }];

      };
    };
}
