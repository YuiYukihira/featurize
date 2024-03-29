{ inputs, cell }:
let
  inherit (inputs) nixpkgs std;
  l = nixpkgs.lib // builtins;
in
{
  auth = std.lib.ops.mkOCI {
    name = "registry.fly.io/featurize-auth";
    tag = cell.args.crateName.version;
    entrypoint = cell.packages.auth;
  };
}
