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
          treefmt = {
            run =
              "${nixpkgs.treefmt}/bin/treefmt --fail-on-change {staged_files}";
          };
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
          invalidLastCharacters = ".,!?";
        };
        body = { required = false; };
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
          scopes = [ "main" "deps" ];
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
          excludes = [ "**/CHANGELOG.md" ];
        };
        cargo-fmt = {
          command = "rustfmt";
          options = [ "--edition" "2021" ];
          includes = [ "*.rs" ];
        };
      };
    };

    packages = [
      nixpkgs.nixpkgs-fmt
      nixpkgs.nodePackages.prettier
      inputs.cells.rust.toolchain.rust
    ];
  };
}
