{ inputs, cell }:
let
  inherit (inputs) nixpkgs std;
  l = nixpkgs.lib // builtins;
in
{
  kratos-config = {
    data = {
      ciphers.algorithm = "xchacha20-poly1305";
      feature_flags.use_continue_with_transitions = true;
      hashers = {
        algorithm = "bcrypt";
        bcrypt.cost = 12;
      };
      identity = {
        default_schema_id = "default";
        schemas = [{
          id = "default";
          url = "file:///identities/identity.schema.json";
        }];
        log = {
          format = "text";
          leak_sensitive_value = false;
          level = "info";
        };
        selfservice = {
          allowed_return_urls = [ "https://auth.dragnof.pro" ];
          default_browser_return_url = "https://featurize.dragnof.pro";
          flows = {
            error.ui_url = "https://auth.dragnof.pro/error";
            login = {
              lifespan = "10m";
              ui_url = "https://auth.dragnof.pro/login";
            };
            logout.after.default_browser_return_url =
              "https://featurize.dragnof.pro";
            recovery = {
              enabled = true;
              ui_url = "https://auth.dragnof.pro/recovery";
              use = "code";
            };
            registration = {
              after.password.hooks =
                [{ hook = "session"; } { hook = "show_verification_ui"; }];
              lifespan = "10m";
              ui_url = "https://auth.dragnof.pro/registration";
            };
            settings = {
              privileged_session_max_age = "15m";
              required_all = "highest_available";
              ui_url = "https://auth.dragnof.pro/settings";
            };
            verification = {
              after.default_browser_return_url =
                "https://featurize.dragnof.pro";
              enabled = true;
              ui_url = "htps://auth.dragnof.pro/verification";
              use = "code";
            };
          };
          methods = {
            code.enabled = true;
            link.enabled = true;
            lookup_secret.enabled = true;
            password.enabled = true;
            totp = {
              config.issuer = "Featurize";
              enabled = true;
            };
          };
        };
      };
      cookies.domain = "dragnof.pro";
      session.cookie.domain = "dragnof.pro";
      courier.smtp = {
        from_address = "auth@dragnof.pro";
        from_name = "Featurize";
      };
      serve.public = {
        base_url = "https://kratos.dragnof.pro/";
        cors = {
          allowed_origins = [ "https://auth.dragnof.pro" ];
          enabled = true;
        };
      };
      oauth2_provider.url = "https://hydra.dragnof.pro";
      version = "v0.13.0";
    };
    output = "deployments/kratos/config.yaml";
  };

  kratos-schema-default = {
    data = {
      "$id" = "https://kratos.dragnof.pro/identity.schema.json";
      "$schema" = "http://json-schema.org/draft-07/schema#";
      title = "Person";
      type = "object";
      properties = {
        traits = {
          type = "object";
          properties = {
            email = {
              type = "string";
              format = "email";
              title = "E-Mail";
              minLength = 3;
              maxLength = 320;
              "ory.sh/kratos" = {
                credentials.password.identifier = true;
                verification.via = "email";
                recovery.via = "email";
              };
            };
            username = {
              title = "Username";
              type = "string";
              minLength = 3;
              maxLength = 30;
              regex = "^[a-zA-Z][a-zA-Z0-9_-]+[a-zA-Z0-9]";
              "ory.sh/kratos".credentials.password.identifier = true;
            };
            name = {
              title = "Full Name";
              type = "string";
            };
          };
          required = [ "email" "username" ];
          additionalProperties = false;
        };
      };
    };
    output = "deployments/kratos/identity.schema.json";
  };

  kratos-fly = {
    data = {
      app = "featurize-kratos";
      primary_region = "lhr";
      build.image = "oryd/kratos:v1.1.0-distroless";
      http_service = {
        internal_port = 4433;
        force_https = true;
        auto_stop_machines = true;
        auto_start_machines = true;
        min_machines_running = 0;
        processes = [ "app" ];
      };

      vm = [{
        memory = "1gb";
        cpu_kind = "shared";
        cpus = 1;
      }];

      files = [
        {
          local_path = "config.yaml";
          guest_path = "/config.yaml";
        }
        {
          local_path = "identity.schema.json";
          guest_path = "/identities/identity.schema.json";
        }
      ];

      deploy.release_command = "migrate sql -e --yes";
      experimental.cmd = [ "serve" "-c" "/config.yaml" "--watch-courier" ];
    };
    output = "deployments/kratos/fly.toml";
  };

  hydra-config = {
    data = {
      log = {
        format = "text";
        level = "info";
      };
      serve.cookies = {
        domain = "dragnof.pro";
        secure = true;
        same_site_mode = "Strict";
      };
      webfinger.oidc_discovery = {
        supported_claims = [ "email" "username" ];
        supported_scope = [ "email" ];
      };
      urls = {
        login = "https://auth.dragnof.pro/login";
        registration = "https://auth.dragnof.pro/registration";
        consent = "https://auth.dragnof.pro/consent";
        logout = "https://auth.dragnof.pro/logout";
        post_logout_redirect = "https://featurize.dragnof.pro";
        identity_provider = {
          publicUrl = "https://kratos.dragnof.pro";
          url = "https://featurize-kratos.internal:4434";
        };
        self = { issuer = "https://hydra.dragnof.pro"; };
      };
      strategies = {
        access_token = "opaque";
        jwt = { scope_claim = "list"; };
        scope = "exact";
      };
      ttl = {
        access_token = "1h";
        refresh_token = "720h";
        id_token = "1h";
        auth_code = "10m";
        login_consent_request = "30m";
      };
      oauth2 = {
        session.encrypt_at_rest = true;
        allowed_top_level_claims = [ "username" "email" "user_uuid" ];
        hashers = {
          bcrypt.cost = 12;
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
    };
    output = "deployments/hydra/config.yaml";
  };

  hydra-fly = {
    data = {
      app = "featurize-hydra";
      primary_region = "lhr";
      build.image = "oryd/hydra:v2.2.0-distroless";
      http_service = {
        internal_port = 4444;
        force_https = true;
        auto_stop_machines = true;
        auto_start_machines = true;
        min_machines_running = 0;
        processes = [ "app" ];
      };
      vm = [{ size = "shared-cpu-1x"; }];

      files = [{
        local_path = "config.yaml";
        guest_path = "/config.yaml";
      }];

      deploy.release_command = "migrate sql -e --yes";
      experimental.cmd = [ "serve" "all" "-c" "/config.yaml" ];
    };
    output = "deployments/hydra/fly.toml";
  };
}
