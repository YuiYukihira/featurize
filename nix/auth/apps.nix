{ inputs, cell }:
let
  inherit (inputs) nixpkgs std;
  l = nixpkgs.lib // builtins;
in
{ auth = inputs.flake-utils.lib.mkApp { drv = cell.packages.auth; }; }
