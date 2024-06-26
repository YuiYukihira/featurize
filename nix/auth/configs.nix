{ inputs, cell }:
let
  inherit (inputs) nixpkgs std;
  l = nixpkgs.lib // builtins;
in
{
  fly = {
    data = {
      app = "featurize-auth";
      primary_region = "lhr";
      vm = [{
        cpu_kind = "shared";
        cpus = 1;
        memory = "1gb";
      }];

      build.image =
        "registry.fly.io/featurize-auth:${cell.args.crateName.version}";

      env = {
        KRATOS_DOMAIN = "https://kratos.dragnof.pro";
        HYDRA_DOMAIN = "https://featurize-hydra.internal:4445";
        PORT = "8080";
        RUST_BACKTRACE = "1";
      };

      http_service = {
        auto_start_machines = true;
        auto_stop_machines = true;
        force_https = true;
        internal_port = 8080;
        min_machines_running = 0;
      };
    };
    output = "deployments/auth/fly.toml";
  };
}
