{ inputs, cell }:
let
  inherit (inputs) nixpkgs std;
  l = nixpkgs.lib // builtins;
in
l.mapAttrs (_: std.lib.dev.mkShell) {
  ci = { ... }: {
    name = "featurize cishell";

    imports =
      [ std.std.devshellProfiles.default inputs.helpers.devshellProfiles.base ];

    commands = [{
      package = nixpkgs.writeShellScriptBin "deploy" ''
        set -eux
        echo "Deploying kratos..."
        pushd "$PRJ_ROOT/deployments/kratos"
        ${nixpkgs.flyctl}/bin/fly deploy
        popd
        echo "Deploying hydra..."
        pushd "$PRJ_ROOT/deployments/hydra"
        ${nixpkgs.flyctl}/bin/fly deploy
        popd
        echo "Deploying auth..."
        pushd "$PRJ_ROOT/deployments/auth"
        std //auth/containers/auth:load
        ${nixpkgs.docker}/bin/docker push registry.fly.io/featurize-auth:${inputs.cells.auth.args.crateName.version}
        ${nixpkgs.flyctl}/bin/fly deploy
        popd
        echo "Done!"
      '';
    }];

    nixago = [
      ((std.lib.dev.mkNixago std.lib.cfg.lefthook) cell.configs.lefthook)
      (std.lib.dev.mkNixago cell.configs.prettier)
      ((std.lib.dev.mkNixago std.lib.cfg.treefmt) cell.configs.treefmt)
      ((std.lib.dev.mkNixago std.lib.cfg.conform) cell.configs.conform)
      ((std.lib.dev.mkNixago inputs.cells.auth.configs.fly))
      ((std.lib.dev.mkNixago inputs.cells.ory.configs.kratos-config))
      ((std.lib.dev.mkNixago inputs.cells.ory.configs.kratos-schema-default))
      ((std.lib.dev.mkNixago inputs.cells.ory.configs.kratos-fly))
      ((std.lib.dev.mkNixago inputs.cells.ory.configs.hydra-config))
      ((std.lib.dev.mkNixago inputs.cells.ory.configs.hydra-fly))
    ];
  };

  default = { ... }: {
    name = "featurize devshell";

    imports = [
      std.std.devshellProfiles.default
      inputs.helpers.devshellProfiles.base
      inputs.helpers.devshellProfiles.language.rust
      inputs.helpers.devshellProfiles.language.c
      inputs.helpers.devshellProfiles.services.minio
      inputs.helpers.devshellProfiles.services.postgres
      inputs.cells.ory.devshellProfiles.hydra
      inputs.cells.ory.devshellProfiles.kratos
      inputs.cells.mailslurper.devshellProfiles.mailslurper
      inputs.cells.auth.devshellProfiles.auth-watch
    ];

    language.rust.packageSet = inputs.cells.rust.toolchain.rust;

    commands = [
      { package = nixpkgs.deno; }
      { package = nixpkgs.tailwindcss; }
      { package = nixpkgs.flyctl; }
      { package = nixpkgs.nodePackages.prettier; }
      {
        package = nixpkgs.writeShellScriptBin "deploy" ''
          set -eux
          args=("$@")
          if [[ "$1" == "all" ]]; then
            args=("auth" "kratos")
          fi

          for app in "''${args[@]}"
          do
            case "$app" in
              auth)
                std //auth/containers/auth:load
                ${nixpkgs.docker}/bin/docker push registry.fly.io/featurize-auth:${inputs.cells.auth.args.crateName.version}
                pushd "$PRJ_ROOT"/deployments/auth
                ${nixpkgs.flyctl}/bin/fly deploy
                popd
                ;;
              kratos)
                pushd "$PRJ_ROOT"/deployments/kratos
                ${nixpkgs.flyctl}/bin/fly deploy
                popd
                ;;
              *)
                echo "No deployment with that name found"
                exit 1
                ;;
            esac
          done'';
      }
    ];

    services = {
      auth-watch = {
        enable = true;
        sentry_dsn =
          "https://27b90b52936e6487f9baffd388628165@o4506987579047936.ingest.us.sentry.io/4506991920807936";
      };
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
        config = {
          log = {
            format = "text";
            level = "info";
          };
          serve.cookies = {
            same_site_mode = "Lax";
            domain = "localhost";
            secure = false;
          };
          webfinger = {
            oidc_discovery = {
              supported_claims = [ "email" "username" ];
              supported_scope = [ "email" ];
            };
          };
          urls = {
            self.issuer = "http://localhost:4444";
            consent = "http://localhost:3000/consent";
            login = "http://localhost:3000/login";
            logout = "http://localhost:3000/logout";
            post_logout_redirect = "http://localhost:3000/";
            registration = "http://localhost:3000/registration";
            identity_provider = {
              publicUrl = "http://localhost:4433";
              url = "http://localhost:4434";
            };
          };
          strategies = {
            access_token = "opaque";
            jwt.scope_claim = "list";
            scope = "exact";
          };
          ttl = {
            access_token = "10m";
            refresh_token = "1h";
            auth_code = "10m";
            login_consent_request = "30m";
          };
          oauth2 = {
            session.encrypt_at_rest = false;
            allowed_top_level_claims = [ "username" "email" "user_uuid" ];
            hashers = {
              bcrypt.cost = 10;
              algorithm = "bcrypt";
            };
            client_credentials.default_grant_allowed_scope = false;
            grant.jwt = {
              iat_optional = false;
              max_ttl = "1h";
              jti_optional = false;
            };
          };
          sqa.opt_out = true;
          version = "v2.2.0";
          secrets = {
            system = [ "a super duper secret" ];
            cookie = [ "a super duper cookie secret" ];
          };
        };
      };
      kratos = {
        enable = true;
        mail.port = 2525;
        config.data = {
          version = "v0.13.0";
          serve = {
            public = {
              base_url = "http://localhost:4433/";
              cors = { enabled = true; };
            };
            admin = { base_url = "http://localhost:4434/"; };
          };
          selfservice = {
            default_browser_return_url = "http://localhost:3000/";
            allowed_return_urls = [
              "http://localhost:3000"
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
              error = { ui_url = "http://localhost:3000/error"; };
              settings = {
                ui_url = "http://localhost:3000/settings";
                privileged_session_max_age = "15m";
                required_aal = "highest_available";
              };
              recovery = {
                enabled = true;
                ui_url = "http://localhost:3000/recovery";
                use = "code";
              };
              verification = {
                enabled = true;
                ui_url = "http://localhost:3000/verification";
                use = "code";
                after = {
                  default_browser_return_url = "http://localhost:3000/";
                };
              };
              logout = {
                after = {
                  default_browser_return_url = "http://localhost:3000/login";
                };
              };
              login = {
                ui_url = "http://localhost:3000/login";
                lifespan = "10m";
              };
              registration = {
                lifespan = "10m";
                ui_url = "http://localhost:3000/registration";
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
                "file://${inputs.self}/deployments/kratos/identity.schema.json";
            }];
          };
          oauth2_provider.url = "http://localhost:4445";
          feature_flags = { use_continue_with_transitions = true; };
        };
      };
    };

    nixago = [
      ((std.lib.dev.mkNixago std.lib.cfg.lefthook) cell.configs.lefthook)
      (std.lib.dev.mkNixago cell.configs.prettier)
      ((std.lib.dev.mkNixago std.lib.cfg.treefmt) cell.configs.treefmt)
      ((std.lib.dev.mkNixago std.lib.cfg.conform) cell.configs.conform)
      ((std.lib.dev.mkNixago inputs.cells.auth.configs.fly))
      ((std.lib.dev.mkNixago inputs.cells.ory.configs.kratos-config))
      ((std.lib.dev.mkNixago inputs.cells.ory.configs.kratos-schema-default))
      ((std.lib.dev.mkNixago inputs.cells.ory.configs.kratos-fly))
      ((std.lib.dev.mkNixago inputs.cells.ory.configs.hydra-config))
      ((std.lib.dev.mkNixago inputs.cells.ory.configs.hydra-fly))
    ];
  };
}
