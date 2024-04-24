{ inputs, cell }:
let
  inherit (inputs) nixpkgs std;
  l = nixpkgs.lib // builtins;
in
{
  featurize = inputs.flake-utils.lib.mkApp {
    drv = nixpkgs.writeShellScriptBin "featurize" ''
      export TEMPLATES_DIR=${cell.packages.featurize}/share/templates
      export PUBLIC_DIR=${cell.packages.featurize}/share/public
      ${cell.packages.featurize}/bin/featurize
    '';
  };
}
