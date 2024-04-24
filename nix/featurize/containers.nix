{ inputs, cell }:
let
  inherit (inputs) nixpkgs std;
  l = nixpkgs.lib // builtins;
in
{
  featurize = std.lib.ops.mkOCI {
    name = "registry.fly.io/featurize";
    tag = cell.args.crateName.version;
    entrypoint = cell.packages.featurize;
  };
}
