{ inputs, cell }: {
  hydra = { config, lib, pkgs, ... }:
    with lib;
    let
      cfg = config.services.hydra;
      dbHost =
        if cfg.db.host == "nix-service" then "localhost" else cfg.db.host;
      start-command = pkgs.writeShellScriptBin "start-hydra" ''
        export DSN="postgres://${cfg.db.user.name}:${cfg.db.user.password}@${dbHost}:${cfg.db.port}/${cfg.db.name}?sslmode=disable&max_conns=20&max_idle_conns=4"
        sleep 5
        ${cfg.package}/bin/hydra migrate sql --config ${cfg.config.location} -e --yes
        ${cfg.package}/bin/hydra serve all --config ${cfg.config.location} --dev
      '';

      configFile = {
        output = cfg.config.location;
        data = cfg.config.data;
      };
    in
    {
      options.services.hydra = {
        enable = mkEnableOption "Enable the service";
        package = mkOption {
          type = types.package;
          default = cell.packages.hydra;
          description = "Package to use";
        };
        db = {
          user = {
            name = mkOption {
              type = types.str;
              default = "hydra";
              description = "User to log into the DB as";
            };
            password = mkOption {
              type = types.str;
              default = "hydra";
              description = "User's password";
            };
          };
          name = mkOption {
            type = types.str;
            default = "hydra";
            description = "database to use";
          };
          host = mkOption {
            type = types.str;
            default = "nix-service";
            description = ''
              Which DB to connect to.
              Set to `nix-service` to automatically start postgres with this service.
            '';
          };
          port = mkOption {
            type = types.str;
            default = "5432";
            description = "Port of the DB host";
          };
        };
        config = {
          location = mkOption {
            type = types.str;
            default = "hydra.yaml";
            description = "Location to generate hydra config file";
          };
          data = mkOption {
            type = types.attrs;
            description = "Config data";
          };
        };
      };

      config = {
        __services.hydra = {
          command = "${start-command}/bin/start-hydra";
          enable = cfg.enable;
          depends = mkIf (cfg.db.host == "nix-service") [ "postgres" ];
        };
        services.postgres.dbs = mkIf (cfg.db.host == "nix-service") [{
          name = cfg.db.name;
          extraSetup = ''
            CREATE USER ${cfg.db.user.name} WITH PASSWORD '${cfg.db.user.password}';
            GRANT ALL PRIVILEGES ON DATABASE ${cfg.db.name} TO ${cfg.db.user.name};
            GRANT ALL ON SCHEMA public TO ${cfg.db.user.name};
            GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO ${cfg.db.user.name};
            GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO ${cfg.db.user.name};
          '';
        }];

        commands = [{ package = cell.packages.hydra; }];

        nixago = mkIf cfg.enable [ (inputs.std.lib.dev.mkNixago configFile) ];
      };
    };
  kratos = { config, lib, pkgs, ... }:
    with lib;
    let
      cfg = config.services.kratos;
      start-command = pkgs.writeShellScriptBin "start-kratos" ''
        ${cfg.package}/bin/kratos serve all --config ${cfg.config.location}
      '';

      configFile = {
        output = cfg.config.location;
        data = cfg.config.data // {
          courier = {
            smtp = {
              connection_uri =
                let
                  host =
                    if cfg.mail.host == "nix-service" then
                      "localhost"
                    else
                      cfg.mail.host;
                  port = toString cfg.mail.port;
                in
                "smtps://${cfg.mail.user.name}:${cfg.mail.user.password}@${host}:${port}/?skip_ssl_verify=true";
            };
          };
        };
      };
    in
    {
      options.services.kratos = {
        enable = mkEnableOption "Enable the service";
        package = mkOption {
          type = types.package;
          default = cell.packages.kratos;
          description = "Package to use";
        };
        db = {
          user = {
            name = mkOption {
              type = types.str;
              default = "kratos";
              description = "User to log into the DB as";
            };
            password = mkOption {
              type = types.str;
              default = "kratos";
              description = "User's password";
            };
          };
          name = mkOption {
            type = types.str;
            default = "kratos";
            description = "database to use";
          };
          host = mkOption {
            type = types.str;
            default = "nix-service";
            description = ''
              Which DB to connect to.
              Set to `nix-service` to automatically start postggres with this service.
            '';
          };
          port = mkOption {
            type = types.str;
            default = "5432";
            description = "Port of the DB host";
          };
        };
        config = {
          location = mkOption {
            type = types.str;
            default = "kratos.yaml";
            description = "Location to generate kratos config file";
          };
          data = mkOption {
            type = types.attrs;
            description = "Config data";
          };
        };
        mail = {
          host = mkOption {
            type = types.str;
            default = "nix-service";
          };
          port = mkOption {
            type = types.int;
            default = 25;
          };
          user = {
            name = mkOption {
              type = types.str;
              default = "test";
            };
            password = mkOption {
              type = types.str;
              default = "test";
            };
          };
        };
      };

      config = {
        __services.kratos = {
          command = "${start-command}/bin/start-kratos";
          enable = cfg.enable;
          depends = mkIf (cfg.db.host == "nix-service") [ "postgres" ];
        };
        services.postgres.dbs = mkIf (cfg.db.host == "nix-service") [{
          name = cfg.db.name;
          extraSetup = ''
            CREATE USER ${cfg.db.user.name} WITH PASSWORD '${cfg.db.user.password}';
            GRANT ALL PRIVEGES ON DATABASE ${cfg.db.name} TO ${cfg.db.user.name};
            GRANT ALL ON SCHEMA public TO ${cfg.db.user.name};
            GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO ${cfg.db.user.name};
            GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO ${cfg.db.user.name};
          '';
        }];

        commands = [{ package = cell.packages.kratos; }];

        nixago = mkIf cfg.enable [ (inputs.std.lib.dev.mkNixago configFile) ];
      };
    };
}
