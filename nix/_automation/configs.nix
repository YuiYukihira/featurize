{ inputs, cell }:
let
  inherit (inputs) nixpkgs std;
  l = nixpkgs.lib // builtins;
in
{
  lefthook = {
    data = {
      commit-msg = {
        commands = {
          conform = {
            run =
              "${nixpkgs.conform}/bin/conform enforce --commit-msg-file {1}";
          };
        };
      };
      pre-commit = {
        commands = {
          treefmt = { run = "${nixpkgs.treefmt}/bin/treefmt {staged_files}"; };
        };
      };
    };
  };

  conform = {
    data = {
      commit = {
        header = {
          length = 89;
          imperative = true;
          case = "upper";
          invalidLastCharacters = ".,!?";
        };
        body = { required = true; };
        dco = true;
        spellcheck = { locale = "US"; };
        conventional = {
          types = [
            "build"
            "chore"
            "ci"
            "docs"
            "feat"
            "fix"
            "perf"
            "refactor"
            "style"
            "test"
            "wip"
          ];
          descriptionLength = 72;
        };
      };
    };
  };

  prettier = {
    data = {
      printWidth = 80;
      proseWrap = "always";
    };
    output = ".prettierrc";
    format = "json";
  };

  treefmt = {
    data = {
      formatter = {
        nix = {
          command = "nixpkgs-fmt";
          includes = [ "*.nix" ];
        };
        prettier = {
          command = "prettier";
          options = [ "--write" ];
          includes = [ "*.md" ];
        };
      };
    };

    packages = [ nixpkgs.nixpkgs-fmt nixpkgs.nodePackages.prettier ];
  };

  auth-fly = {
    data = {
      app = "featurize-auth";
      primary_region = "lhr";

      http_service = {
        internal_port = 8080;
        force_https = true;
        auto_stop_machines = true;
        auto_start_machines = true;
        min_machines_running = 0;
      };

      vm = [{
        memory = "1gb";
        cpu_kind = "shared";
        cpus = 1;
      }];

      build = {
        image =
          "registry.fly.io/featurize-auth:${inputs.cells.auth.args.crateName.version}";
      };

      env = {
        KRATOS_DOMAIN =
          "https://flamboyant-austin-06hwmvtz98.projects.oryapis.com";
        PORT = "8080";
      };
    };
    output = "auth/fly.toml";
  };
}
