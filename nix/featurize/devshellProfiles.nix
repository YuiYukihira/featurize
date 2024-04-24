{ inputs, cell }: {
  featurize-watch = { config, lib, pkgs, ... }:
    with lib;
    let
      cfg = config.services.featurize-watch;
      start-command = pkgs.writeShellScriptBin "featurize" ''
        export SENTRY_DSN="${cfg.sentry_dsn}"
        export KRATOS_DOMAIN="http://localhost:4433"
        export HYDRA_DOMAIN="http://localhost:4445"
        export PORT="${toString cfg.port}"

        sigint_handler()
        {
          kill $PID
          exit
        }

        trap sigint_handler SIGINT

        pushd $PRJ_ROOT
        while true; do
          ${inputs.cells.rust.toolchain.rust}/bin/cargo run &
          PID=$!
          ${pkgs.inotify-tools}/bin/inotifywait -e modify -e create -e delete -e attrib -r src public templates
          kill $PID
        done
      '';

      tailwind-watch = pkgs.writeShellScriptBin "tailwind" ''
        set -eux
        pushd $PRJ_ROOT
        ${pkgs.tailwindcss}/bin/tailwindcss -i $PRJ_ROOT/src/inputs.css -o $PRJ_ROOT/public/output.css --watch=always
      '';
    in
    {
      options.services.featurize-watch = {
        enable = mkEnableOption "Enable the service";
        sentry_dsn = mkOption {
          type = types.str;
          description = "Sentry DSN";
        };
        port = mkOption {
          type = types.int;
          default = 3001;
          description = "Port to expose on";
        };
      };

      config = {
        __services.featurize-watch = {
          command = "${start-command}/bin/featurize";
          enable = cfg.enable;
          depends = [ "kratos" "hydra" "auth-watch" "featurize-tailwind" ];
        };

        __services.featurize-tailwind = {
          command = "${tailwind-featurize}/bin/tailwind";
          enable = true;
          depends = [ ];
        };
      };
    };
}
