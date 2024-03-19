{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    std = {
      url = "github:divnix/std";
      inputs.devshell.url = "github:numtide/devshell";
      inputs.nixago.url = "github:nix-community/nixago";
      inputs.n2c.url = "github:nlewo/nix2container";
    };
    helpers.url = "sourcehut:~yuiyukihira/devshell";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { std, ... }@inputs:
    std.growOn
      {
        inherit inputs;
        cellsFrom = ./nix;
        cellBlocks = [
          (std.blockTypes.runnables "apps")
          (std.blockTypes.installables "packages")
          (std.blockTypes.devshells "devshells")
          (std.blockTypes.nixago "configs")
          (std.blockTypes.functions "toolchain")
          (std.blockTypes.functions "args")
          (std.blockTypes.containers "containers")
          (std.blockTypes.functions "devshellProfiles")
        ];
      }
      {
        packages = std.harvest inputs.self [ [ "ory" "packages" ] ];
        devShells = std.harvest inputs.self [ [ "_automation" "devshells" ] ];
        devshellProfiles =
          std.harvest inputs.self [ [ "featurize" "devshellProfiles" ] ];
      };
}
