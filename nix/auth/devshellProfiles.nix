{ inputs, cell }: {
  auth-watch = { config, lib, pkgs, ... }:
    with lib;
    let
      cfg = config.services.auth-watch;
      start-command = pkgs.writeShellScriptBin "auth" ''
        set -eux
        export SENTRY_DSN="${cfg.sentry_dsn}"
        export KRATOS_DOMAIN="http://localhost:4433"
        export COOKIE_DOMAIN="localhost"
        export COOKIE_SECRET="$(tr -dc A-Za-z0-9 </dev/urandom | head -c 32)"

        sigint_handler()
        {
          kill $PID
          exit
        }

        trap sigint_handler SIGINT

        pushd $PRJ_ROOT/auth
        while true; do
          PORT="${
            toString cfg.port
          }" ${inputs.cells.rust.toolchain.rust}/bin/cargo run &
          PID=$!
          ${pkgs.inotify-tools}/bin/inotifywait -e modify -e create -e delete -e attrib -r src public templates
          kill $PID
        done
      '';

      tailwind-watch = pkgs.writeShellScriptBin "tailwind" ''
        set -eux
        pushd $PRJ_ROOT/auth
        ${pkgs.tailwindcss}/bin/tailwindcss -i $PRJ_ROOT/auth/src/input.css -o $PRJ_ROOT/auth/public/output.css --watch=always
      '';
    in {
      options.services.auth-watch = {
        enable = mkEnableOption "Enable the service";
        sentry_dsn = mkOption {
          type = types.str;
          description = "Sentry DSN";
        };
        port = mkOption {
          type = types.int;
          default = 3000;
          description = "Port to expose on";
        };
      };

      config = {
        __services.auth-watch = {
          command = "${start-command}/bin/auth";
          enable = cfg.enable;
          depends = [ "kratos" "hydra" "mailslurper" "auth-tailwind" ];
        };

        __services.auth-tailwind = {
          command = "${tailwind-watch}/bin/tailwind";
          enable = true;
          depends = [ ];
        };
      };
    };
}
