{ inputs, cell }:
let
  inherit (inputs) nixpkgs std;
  l = nixpkgs.lib // builtins;
in
l.mapAttrs (_: std.lib.dev.mkShell) {
  default = { ... }: {
    name = "romanticise devshell";

    imports = [
      std.std.devshellProfiles.default
      inputs.helpers.devshellProfiles.base
      inputs.helpers.devshellProfiles.services.minio
      inputs.helpers.devshellProfiles.services.postgres
      inputs.cells.ory.devshellProfiles.hydra
      inputs.cells.ory.devshellProfiles.kratos
      inputs.cells.mailslurper.devshellProfiles.mailslurper
    ];

    commands = [ ];

    services = {
      mailslurper = { enable = true; };
      minio = {
        enable = true;
        rootUser = {
          name = "minio";
          password = "minio-password";
        };
      };
      hydra = {
        enable = true;
        config.data = {
          serve.cookies.same_site_mode = "Lax";
          urls = {
            self.issuer = "http://127.0.0.1:4444";
            consent = "http://127.0.0.1:3000/consent";
            login = "http://127.0.0.1:3000/login";
            logout = "http://127.0.0.1:3000/logout";
          };
          secrets.system = [ "a super duper secret" ];

          oidc.subject_identifiers = {
            supported_types = [ "pairwise" "public" ];
            pairwise.salt = "a super duper good salt";
          };
        };
      };
      kratos = {
        enable = true;
        config.data = {
          version = "v0.13.0";
          dsn = "memory";
          serve = {
            public = {
              base_url = "http://localhost:4433/";
              cors = { enabled = true; };
            };
            admin = { base_url = "http://localhost:4434/"; };
          };
          selfservice = {
            default_browser_return_url = "http://localhost:4455/";
            allowed_return_urls = [
              "http://localhost:4455"
              "http://localhost:19006/Callback"
              "exp://localhost:8081/--/Callback"
            ];
            methods = {
              password = { enabled = true; };
              totp = {
                config = { issuer = "Kratos"; };
                enabled = true;
              };
              lookup_secret = { enabled = true; };
              link = { enabled = true; };
              code = { enabled = true; };
            };
            flows = {
              error = { ui_url = "http://localhost:4455/error"; };
              settings = {
                ui_url = "http://localhost:4455/settings";
                privileged_session_max_age = "15m";
                required_aal = "highest_available";
              };
              recovery = {
                enabled = true;
                ui_url = "http://localhost:4455/recovery";
                use = "code";
              };
              verification = {
                enabled = true;
                ui_url = "http://localhost:4455/verification";
                use = "code";
                after = {
                  default_browser_return_url = "http://localhost:4455/";
                };
              };
              logout = {
                after = {
                  default_browser_return_url = "http://localhost:4455/login";
                };
              };
              login = {
                ui_url = "http://localhost:4455/login";
                lifespan = "10m";
              };
              registration = {
                lifespan = "10m";
                ui_url = "http://localhost:4455/registration";
                after = {
                  password = {
                    hooks = [
                      { hook = "session"; }
                      { hook = "show_verification_ui"; }
                    ];
                  };
                };
              };
            };
          };
          log = {
            level = "debug";
            format = "text";
            leak_sensitive_values = true;
          };
          secrets = {
            cookie = [ "PLEASE-CHANGE-ME-I-AM-VERY-INSECURE" ];
            cipher = [ "32-LONG-SECRET-NOT_SECURE-AT-ALL" ];
          };
          ciphers = { algorithm = "xchacha20-poly1305"; };
          hashers = {
            algorithm = "bcrypt";
            bcrypt = { cost = 8; };
          };
          identity = {
            default_schema_id = "default";
            schemas = [{
              id = "default";
              url =
                "https://github.com/ory/kratos/raw/master/contrib/quickstart/kratos/email-password/identity.schema.json";
            }];
          };
          feature_flags = { use_continue_with_transitions = true; };
        };
      };
    };

    nixago = [
      ((std.lib.dev.mkNixago std.lib.cfg.lefthook) cell.configs.lefthook)
      (std.lib.dev.mkNixago cell.configs.prettier)
      ((std.lib.dev.mkNixago std.lib.cfg.treefmt) cell.configs.treefmt)
      ((std.lib.dev.mkNixago std.lib.cfg.conform) cell.configs.conform)
    ];
  };
}
