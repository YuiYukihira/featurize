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
}
